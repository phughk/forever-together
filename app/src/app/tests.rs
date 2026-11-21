use super::memnet::{MemNetHandle, MemNetwork, NetCall};
use super::*;
use crate::code::test_encoder::{encode, sample_character};
use crate::engine::state::MemberView;
use crate::store::MemStore;

fn peer(i: u8) -> PeerId {
    PeerId([i; 32])
}

fn core_with(store: Arc<dyn Store>) -> (LiveCore, MemNetHandle, watch::Sender<NetworkView>) {
    let (net, handle) = MemNetwork::new();
    let (tx, rx) = watch::channel(NetworkView {
        peers: 0,
        bootstrap: BootstrapSource::None,
    });
    let core = LiveCore::spawn(
        CoreDeps {
            store,
            me: peer(1),
            clock: system_clock(),
            net_status: rx,
        },
        move || async move { Ok::<_, NetError>(net) },
    );
    (core, handle, tx)
}

fn new_core() -> (LiveCore, MemNetHandle, watch::Sender<NetworkView>) {
    core_with(Arc::new(MemStore::new()))
}

#[tokio::test]
async fn character_codes_are_decoded_stored_and_survive_a_restart() {
    let store: Arc<dyn Store> = Arc::new(MemStore::new());
    let (core, _h, _tx) = core_with(store.clone());
    let c = core
        .set_character(encode(&sample_character()))
        .await
        .unwrap();
    assert_eq!(
        (c.first_name.as_str(), c.surname.as_str()),
        ("Aldric", "Thornwood")
    );
    assert_eq!(c.role, RoleView::Tank);
    assert_eq!(c.faction, FactionView::Alliance);
    assert_eq!(c.class_name.as_deref(), Some("Warrior"));
    assert!(c.zone_known);
    assert_eq!(c.created_at, 29_000_000 * 60_000);
    assert_eq!(core.state().character.unwrap(), c);

    let err = core.set_character("WFT1:ZZZ".into()).await.unwrap_err();
    assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        ["BadText", "BadCheck", "Truncated"].contains(&err.code.as_str()),
        "{}",
        err.code
    );
    assert_eq!(
        core.set_character("hello".into()).await.unwrap_err().code,
        "BadText"
    );
    assert_eq!(
        core.set_character("WFT2:AAAA".into())
            .await
            .unwrap_err()
            .code,
        "BadText"
    );
    // A refused code leaves the character alone.
    assert!(core.state().character.is_some());

    let (again, _h2, _tx2) = core_with(store);
    assert_eq!(again.state().character.unwrap().first_name, "Aldric");
    again.clear_character().await.unwrap();
    assert!(again.state().character.is_none());
}

#[tokio::test]
async fn search_lifecycle_joins_topics_and_reports_state() {
    let (core, net, _tx) = new_core();
    let err = core
        .start_search(vec!["the-deadmines".into()])
        .await
        .unwrap_err();
    assert_eq!(err.code, "no_character");

    core.set_character(encode(&sample_character()))
        .await
        .unwrap();
    let d = core.dungeons().await;
    assert!(d.dungeons.iter().any(|x| x.slug == "the-deadmines"));
    assert!(d.recommended.contains(&"the-deadmines".to_string()));

    assert_eq!(
        core.start_search(vec![]).await.unwrap_err().code,
        "no_dungeons"
    );
    assert_eq!(
        core.start_search(vec!["nowhere".into()])
            .await
            .unwrap_err()
            .code,
        "unknown_dungeon"
    );
    // Coming-soon dungeons cannot be searched.
    assert_eq!(
        core.start_search(vec!["hall-of-thanes".into()])
            .await
            .unwrap_err()
            .code,
        "unknown_dungeon"
    );
    assert!(net.calls().is_empty());

    core.start_search(vec!["the-deadmines".into(), "the-deadmines".into()])
        .await
        .unwrap();
    let s = core.state();
    assert_eq!(s.search.status, SearchStatus::Searching);
    assert_eq!(s.search.dungeons, ["the-deadmines"]);
    assert_eq!(s.search.stage, 0);
    assert_eq!(s.search.topics, 1);
    assert!(s.search.started_at.unwrap() > 1_000_000_000_000);
    let stage_clock = s.search.stage_clock_started_at.unwrap();
    assert!(stage_clock <= s.server_now && s.server_now - stage_clock < 60_000);
    assert_eq!(s.search.searching, Some(0));
    assert!(s.party.is_none());
    let topic = crate::dungeons::topic_id(3, "5", code::Faction::Alliance, "the-deadmines", 0);
    assert!(net.calls().contains(&NetCall::Join(topic)));

    // Nothing to confirm while searching.
    assert_eq!(core.confirm_party().await.unwrap_err().code, "no_party");
    assert_eq!(core.decline_party().await.unwrap_err().code, "no_party");

    core.stop_search().await.unwrap();
    let s = core.state();
    assert_eq!(s.search.status, SearchStatus::Idle);
    assert!(s.search.dungeons.is_empty() && s.search.started_at.is_none());
    assert!(net.calls().contains(&NetCall::Leave(topic)));
    // Stopping twice is harmless.
    core.stop_search().await.unwrap();
}

#[tokio::test]
async fn a_character_without_a_role_cannot_search() {
    let (core, _net, _tx) = new_core();
    let mut c = sample_character();
    c.role = None;
    core.set_character(encode(&c)).await.unwrap();
    assert_eq!(core.state().character.unwrap().role, RoleView::Unknown);
    let e = core
        .start_search(vec!["the-deadmines".into()])
        .await
        .unwrap_err();
    assert_eq!(e.code, "role_unknown");
}

#[tokio::test]
async fn replacing_the_character_stops_the_search() {
    let (core, net, _tx) = new_core();
    core.set_character(encode(&sample_character()))
        .await
        .unwrap();
    core.start_search(vec!["the-deadmines".into()])
        .await
        .unwrap();
    let mut other = sample_character();
    other.first_name = "Brin".into();
    core.set_character(encode(&other)).await.unwrap();
    assert_eq!(core.state().search.status, SearchStatus::Idle);
    assert!(net.calls().iter().any(|c| matches!(c, NetCall::Leave(_))));
    assert_eq!(core.state().character.unwrap().first_name, "Brin");
}

#[tokio::test]
async fn settings_are_validated_saved_and_published() {
    let store: Arc<dyn Store> = Arc::new(MemStore::new());
    let (core, _h, _tx) = core_with(store.clone());
    assert_eq!(
        core.settings(),
        SettingsView {
            lan_mode: false,
            known_peers_size: 100,
            trusted_peers: vec![]
        }
    );
    let key = "AB".repeat(32);
    let s = core
        .update_settings(SettingsPatch {
            lan_mode: Some(true),
            known_peers_size: Some(250),
            trusted_peers: Some(vec![key.clone(), key.to_lowercase(), format!(" {key} ")]),
        })
        .await
        .unwrap();
    assert!(s.lan_mode);
    assert_eq!(s.known_peers_size, 250);
    assert_eq!(
        s.trusted_peers,
        [key.to_lowercase()],
        "normalized and de-duplicated"
    );
    assert_eq!(core.state().settings, s);
    assert_eq!(
        Settings::load(store.as_ref()).unwrap().known_peers_cache,
        250
    );

    for bad in [
        SettingsPatch {
            known_peers_size: Some(0),
            ..Default::default()
        },
        SettingsPatch {
            known_peers_size: Some(1_000_000),
            ..Default::default()
        },
        SettingsPatch {
            trusted_peers: Some(vec!["nope".into()]),
            ..Default::default()
        },
        SettingsPatch {
            trusted_peers: Some(vec!["ab".repeat(33)]),
            ..Default::default()
        },
    ] {
        let e = core.update_settings(bad).await.unwrap_err();
        assert_eq!(e.status, StatusCode::BAD_REQUEST);
    }
    // A refused patch changes nothing.
    assert_eq!(core.settings(), s);
    // A partial patch leaves the other fields.
    let s2 = core
        .update_settings(SettingsPatch {
            lan_mode: Some(false),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(!s2.lan_mode && s2.known_peers_size == 250);
}

#[tokio::test]
async fn network_status_changes_reach_the_snapshot_and_subscribers() {
    let (core, _h, tx) = new_core();
    let mut rx = core.subscribe();
    rx.borrow_and_update();
    tx.send(NetworkView {
        peers: 7,
        bootstrap: BootstrapSource::Remembered,
    })
    .unwrap();
    tokio::time::timeout(Duration::from_secs(3), rx.changed())
        .await
        .expect("no update")
        .unwrap();
    let s = rx.borrow().clone();
    assert_eq!(s.network.peers, 7);
    assert_eq!(s.network.bootstrap, BootstrapSource::Remembered);
}

#[test]
fn party_view_maps_the_engine_view() {
    let me = peer(1);
    let c = sample_character();
    let members = |names: bool, confirmed_by_me: bool| -> Vec<MemberView> {
        (1..=5u8)
            .map(|i| MemberView {
                peer: peer(i),
                details: None,
                ready: i != 5,
                confirmed: i == 1 && confirmed_by_me,
                name: names.then(|| (format!("N{i}"), format!("S{i}"))),
            })
            .collect()
    };
    let view = |mode, confirm_prompt, party| View {
        mode,
        stage: 0,
        peers_seen: 0,
        next_stage_in: None,
        confirm_prompt,
        party,
        committed: None,
        last_drop: None,
    };
    let mut track = PartyTrack::default();

    // Searching has no party.
    assert!(
        party_view(
            &view(Mode::Searching, false, vec![]),
            &me,
            Some(&c),
            0,
            &mut track
        )
        .is_none()
    );

    let v = view(Mode::Committed, true, members(false, false));
    let p = party_view(&v, &me, Some(&c), 1_000, &mut track).unwrap();
    assert_eq!(p.phase, PartyPhase::Confirming);
    assert_eq!(p.confirm_deadline, Some(1_000 + 15_000));
    assert_eq!(p.members.len(), 5);
    let mine = p.members.iter().find(|m| m.is_self).unwrap();
    assert_eq!(
        (mine.role, mine.level, mine.item_level),
        (RoleView::Tank, 24, 120)
    );
    assert!(
        p.members.iter().all(|m| m.first_name.is_none()),
        "names stay hidden"
    );
    assert!(
        !p.members
            .iter()
            .find(|m| m.peer_id == peer(5).to_hex())
            .unwrap()
            .ready
    );
    // The deadline is fixed when the party is first seen.
    let again = party_view(&v, &me, Some(&c), 9_000, &mut track).unwrap();
    assert_eq!(again.confirm_deadline, Some(16_000));
    assert_eq!(again.id, p.id);
    // Once I have confirmed there is no deadline.
    let v = view(Mode::Committed, false, members(false, true));
    let p = party_view(&v, &me, Some(&c), 10_000, &mut track).unwrap();
    assert_eq!(p.confirm_deadline, None);
    assert!(p.members.iter().find(|m| m.is_self).unwrap().confirmed);

    // Final: names appear.
    let v = view(Mode::Final, false, members(true, true));
    let p = party_view(&v, &me, Some(&c), 11_000, &mut track).unwrap();
    assert_eq!(p.phase, PartyPhase::Final);
    assert_eq!(p.members[0].first_name.as_deref(), Some("N1"));
    assert_eq!(p.members[0].surname.as_deref(), Some("S1"));
}
