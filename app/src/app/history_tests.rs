//! Search history through the running core (ADR 0027).

use super::memnet::{MemNetHandle, MemNetwork};
use super::*;
use crate::code::test_encoder::{encode, sample_character};
use crate::store::{MemStore, PREFIX_HISTORY};
use std::sync::atomic::{AtomicU64, Ordering};

fn peer(i: u8) -> PeerId {
    PeerId([i; 32])
}

/// A core whose clock the test moves. The core's own one-second ticker still runs in real time,
/// so after moving the clock a test waits a little over a second for a tick to see it.
fn core_with_clock(store: Arc<dyn Store>) -> (LiveCore, Arc<AtomicU64>, MemNetHandle) {
    core_with_peers(store, 0)
}

/// As `core_with_clock`, with `peers` connected peers reported (so a situation other than
/// `not_connected` can be diagnosed).
fn core_with_peers(store: Arc<dyn Store>, peers: u32) -> (LiveCore, Arc<AtomicU64>, MemNetHandle) {
    let (net, handle) = MemNetwork::new();
    let (_tx, rx) = watch::channel(NetworkView {
        peers,
        bootstrap: BootstrapSource::None,
    });
    let offset = Arc::new(AtomicU64::new(0));
    let o = offset.clone();
    let core = LiveCore::spawn(
        CoreDeps {
            store,
            me: peer(1),
            clock: Arc::new(move || Duration::from_secs(1_800_000_000 + o.load(Ordering::SeqCst))),
            net_status: rx,
        },
        move || async move { Ok::<_, NetError>(net) },
    );
    (core, offset, handle)
}

async fn tick() {
    tokio::time::sleep(Duration::from_millis(1300)).await;
}

fn advance(offset: &AtomicU64, secs: u64) {
    offset.fetch_add(secs, Ordering::SeqCst);
}

async fn start_deadmines(core: &LiveCore) {
    core.start_search(vec!["the-deadmines".into()])
        .await
        .unwrap();
}

async fn set_character(core: &LiveCore) {
    core.set_character(encode(&sample_character()))
        .await
        .unwrap();
}

#[tokio::test]
async fn stops_and_expiry_are_recorded_by_the_outcome_rules() {
    let store = Arc::new(MemStore::new());
    let (core, clock, _net) = core_with_clock(store.clone());
    set_character(&core).await;
    start_deadmines(&core).await;
    // A stop at 30 s records nothing.
    advance(&clock, 30);
    core.stop_search().await.unwrap();
    assert!(history::load(store.as_ref()).unwrap().is_empty());
    // A stop at 90 s records a Stopped entry with the first checkpoint only.
    start_deadmines(&core).await;
    advance(&clock, 90);
    tick().await;
    core.stop_search().await.unwrap();
    let rows = history::load(store.as_ref()).unwrap();
    assert_eq!(rows.len(), 1);
    let e = &rows[0].1;
    assert_eq!(e.outcome, history::Outcome::Stopped);
    assert_eq!(e.dungeon, "the-deadmines");
    assert_eq!(
        (e.region, e.realm.as_str(), e.faction.as_str()),
        (3, "5", "alliance")
    );
    assert_eq!(e.own_role, "tank");
    assert!(e.end_utc - e.start_utc >= 90);
    assert!(e.checkpoint_1.is_some() && e.checkpoint_2.is_none());
    // A search that runs to the limit records Expired.
    start_deadmines(&core).await;
    advance(&clock, 605);
    tick().await;
    assert_eq!(core.state().search.status, SearchStatus::LimitReached);
    let rows = history::load(store.as_ref()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].1.outcome, history::Outcome::Expired);
    assert!(rows[1].1.checkpoint_2.is_some());
}

#[tokio::test]
async fn a_character_change_is_not_a_result_and_clearing_the_character_clears_history() {
    let store = Arc::new(MemStore::new());
    let (core, clock, _net) = core_with_clock(store.clone());
    set_character(&core).await;
    start_deadmines(&core).await;
    advance(&clock, 90);
    tick().await;
    // Replacing the character ends the search for the host's reasons: nothing is recorded.
    let mut other = sample_character();
    other.first_name = "Brin".into();
    core.set_character(encode(&other)).await.unwrap();
    assert!(history::load(store.as_ref()).unwrap().is_empty());
    // Record one, then clear the character: the history goes with it.
    start_deadmines(&core).await;
    advance(&clock, 90);
    tick().await;
    core.stop_search().await.unwrap();
    assert_eq!(history::load(store.as_ref()).unwrap().len(), 1);
    core.clear_character().await.unwrap();
    assert!(store.list(PREFIX_HISTORY).unwrap().is_empty());
}

#[tokio::test]
async fn clear_history_deletes_every_entry_and_keeps_the_character() {
    let store = Arc::new(MemStore::new());
    let (core, clock, _net) = core_with_clock(store.clone());
    set_character(&core).await;
    start_deadmines(&core).await;
    advance(&clock, 90);
    tick().await;
    core.stop_search().await.unwrap();
    store.put("history/000000000001", b"junk").unwrap();
    assert_eq!(store.list(PREFIX_HISTORY).unwrap().len(), 2);
    core.clear_history().await.unwrap();
    assert!(store.list(PREFIX_HISTORY).unwrap().is_empty());
    assert!(core.state().character.is_some());
}

#[tokio::test]
async fn a_failing_history_store_never_stops_a_search() {
    let store = Arc::new(history::tests::FailingStore(MemStore::new()));
    let (core, clock, _net) = core_with_clock(store.clone());
    set_character(&core).await;
    start_deadmines(&core).await;
    advance(&clock, 605);
    tick().await;
    // The search ended as normal and the app still answers.
    assert_eq!(core.state().search.status, SearchStatus::LimitReached);
    assert!(store.list(PREFIX_HISTORY).unwrap().is_empty());
    start_deadmines(&core).await;
    advance(&clock, 90);
    tick().await;
    core.stop_search().await.unwrap();
    assert_eq!(core.state().search.status, SearchStatus::Idle);
}

// ---- the outlook (ADR 0027, stage 8b) ---------------------------------------------------

/// Earlier searches by the sample character (a tank) that saw nobody at 55 s (the healer bin):
/// `formed` found a group, `expired` did not. Invented data.
fn seed_history(store: &MemStore, formed: u32, expired: u32) {
    let mut start = 1_799_000_000u64;
    for outcome in std::iter::repeat_n(history::Outcome::Formed, formed as usize).chain(
        std::iter::repeat_n(history::Outcome::Expired, expired as usize),
    ) {
        let mut e = history::tests::entry(start, "the-deadmines");
        e.checkpoint_1 = Some(history::CheckpointEntry {
            counts: history::Counts {
                players: 0,
                tanks: 0,
                healers: 0,
                damage: 0,
            },
            stage: 0,
            sample_full: false,
        });
        e.outcome = outcome;
        store
            .put(&e.key(), &serde_json::to_vec(&e).unwrap())
            .unwrap();
        start += 1_000;
    }
}

fn outlook(core: &LiveCore) -> Option<String> {
    core.state().search.diagnosis.unwrap().outlook
}

#[tokio::test]
async fn the_outlook_appears_at_the_checkpoint_only_with_dependable_history() {
    let store = Arc::new(MemStore::new());
    seed_history(&store, 3, 37);
    let (core, clock, _net) = core_with_peers(store.clone(), 3);
    set_character(&core).await;
    start_deadmines(&core).await;
    // Before the 55 s checkpoint: a diagnosis, but no outlook.
    advance(&clock, 40);
    tick().await;
    let d = core.state().search.diagnosis.unwrap();
    assert_eq!(d.situation, "alone");
    assert_eq!(d.outlook, None);
    // After it, still unformed: a hedged band, and the situation is unchanged.
    advance(&clock, 20);
    tick().await;
    let d = core.state().search.diagnosis.unwrap();
    assert_eq!((d.situation.as_str(), d.ended), ("alone", false));
    assert_eq!(d.outlook.as_deref(), Some("rarely_found"));
    // Clearing the history clears the cache with it.
    core.clear_history().await.unwrap();
    tick().await;
    assert_eq!(outlook(&core), None);
}

#[tokio::test]
async fn too_little_history_gives_no_outlook() {
    let store = Arc::new(MemStore::new());
    seed_history(&store, 2, 2);
    let (core, clock, _net) = core_with_peers(store.clone(), 3);
    set_character(&core).await;
    start_deadmines(&core).await;
    advance(&clock, 60);
    tick().await;
    assert_eq!(outlook(&core), None);
}

#[tokio::test]
async fn the_outlook_is_never_shown_once_the_search_ended() {
    let store = Arc::new(MemStore::new());
    seed_history(&store, 3, 37);
    let (core, clock, _net) = core_with_peers(store.clone(), 3);
    set_character(&core).await;
    start_deadmines(&core).await;
    advance(&clock, 60);
    tick().await;
    assert!(outlook(&core).is_some());
    advance(&clock, 550);
    tick().await;
    assert_eq!(core.state().search.status, SearchStatus::LimitReached);
    assert_eq!(outlook(&core), None);
}

#[tokio::test]
async fn a_recorded_search_joins_the_history_the_estimator_reads() {
    let store = Arc::new(MemStore::new());
    seed_history(&store, 3, 36);
    let (core, clock, _net) = core_with_peers(store.clone(), 3);
    set_character(&core).await;
    // 39 entries are 3 successes and 36 failures; one more expired search is recorded and the
    // cache is refreshed. The recorded entry joins the same pool as the seeded ones.
    start_deadmines(&core).await;
    advance(&clock, 605);
    tick().await;
    assert_eq!(history::load(store.as_ref()).unwrap().len(), 40);
    start_deadmines(&core).await;
    advance(&clock, 60);
    tick().await;
    assert_eq!(outlook(&core).as_deref(), Some("rarely_found"));
}
