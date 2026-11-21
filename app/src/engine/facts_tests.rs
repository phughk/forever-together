//! The facts record the engine emits when a search ends (ADR 0027).

use super::testkit::*;
use super::*;
use std::time::Duration;

fn solo(c: &mut Cluster) {
    start(
        c,
        0,
        profile(Role::Tank, 24, 100, Faction::Alliance, "Testa"),
    );
}

fn five(c: &mut Cluster) {
    let roles = [
        Role::Tank,
        Role::Healer,
        Role::Damage,
        Role::Damage,
        Role::Damage,
    ];
    for (i, r) in roles.iter().enumerate() {
        start(
            c,
            i,
            profile(*r, 24, 100, Faction::Alliance, &format!("N{i}")),
        );
    }
}

#[test]
fn a_solo_tank_to_expiry_gives_one_expired_fact_with_checkpoints() {
    let mut c = Cluster::new(&[1]);
    solo(&mut c);
    c.run(610);
    let ended = &c.nodes[0].ended;
    assert_eq!(ended.len(), 1);
    let f = &ended[0];
    assert_eq!(f.outcome, SearchOutcome::Expired);
    assert_eq!(f.elapsed, crate::config::SEARCH_LIMIT);
    assert_eq!(f.own_role, Role::Tank);
    assert_eq!(f.level, 24);
    assert_eq!(f.topic_count, 1);
    assert_eq!(f.peak, RoleCounts::default());
    assert_eq!(f.stage_reached, 4);
    assert_eq!((f.first_formed_at, f.drops, f.declines), (None, 0, 0));
    // 55 s is before the 60 s step, 115 s before the 120 s step.
    let (a, b) = (f.checkpoint_1.unwrap(), f.checkpoint_2.unwrap());
    assert_eq!((a.stage, b.stage), (0, 1));
    assert_eq!(a.counts, RoleCounts::default());
    assert!(!a.sample_full);
}

#[test]
fn the_live_stats_carry_the_checkpoint_only_once_it_is_taken() {
    let mut c = Cluster::new(&[1]);
    solo(&mut c);
    c.run(50);
    assert_eq!(
        c.nodes[0].engine.view(c.now).stats.unwrap().checkpoint,
        None
    );
    c.run(10);
    let cp = c.nodes[0].engine.view(c.now).stats.unwrap().checkpoint;
    assert_eq!(cp.map(|c| c.counts), Some(RoleCounts::default()));
}

#[test]
fn a_search_that_ends_before_a_checkpoint_stores_none_not_zero() {
    let mut c = Cluster::new(&[1]);
    solo(&mut c);
    c.run(54);
    c.send(0, Event::UserStop);
    let f = &c.nodes[0].ended[0];
    assert_eq!(f.outcome, SearchOutcome::Stopped);
    assert_eq!(f.elapsed, Duration::from_secs(54));
    assert_eq!((f.checkpoint_1, f.checkpoint_2), (None, None));
}

#[test]
fn a_stop_at_90_s_has_the_first_checkpoint_only() {
    let mut c = Cluster::new(&[1]);
    solo(&mut c);
    c.run(90);
    c.send(0, Event::UserStop);
    let f = &c.nodes[0].ended[0];
    assert_eq!(f.outcome, SearchOutcome::Stopped);
    assert_eq!(f.elapsed, Duration::from_secs(90));
    assert!(f.checkpoint_1.is_some());
    assert_eq!(f.checkpoint_2, None);
    // Stopping again, with no search, emits nothing more.
    c.send(0, Event::UserStop);
    assert_eq!(c.nodes[0].ended.len(), 1);
}

#[test]
fn a_restart_ends_the_old_search_as_ignored() {
    let mut c = Cluster::new(&[1]);
    solo(&mut c);
    c.run(20);
    solo(&mut c);
    let ended = &c.nodes[0].ended;
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0].outcome, SearchOutcome::Ignored);
    assert_eq!(ended[0].elapsed, Duration::from_secs(20));
}

#[test]
fn a_decline_then_a_re_form_is_one_formed_entry_and_not_a_failure() {
    let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
    five(&mut c);
    c.run(12);
    c.send(0, Event::UserDecline);
    c.run(10);
    assert_eq!(c.modes(), vec![Mode::Committed; 5]);
    for i in 0..5 {
        c.send(i, Event::UserConfirm);
    }
    assert_eq!(c.modes(), vec![Mode::Final; 5]);
    let ended = &c.nodes[0].ended;
    assert_eq!(ended.len(), 1, "one entry for the whole search");
    let f = &ended[0];
    assert_eq!(f.outcome, SearchOutcome::Formed);
    assert_eq!((f.drops, f.declines), (0, 1));
    assert!(
        f.first_formed_at
            .is_some_and(|t| t <= Duration::from_secs(15))
    );
    // Peak counts are the most seen: the four others.
    assert_eq!(f.peak.players, 4);
}

#[test]
fn a_member_that_goes_silent_is_a_drop_and_the_search_is_still_one_entry() {
    let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
    five(&mut c);
    c.run(12);
    let silent = c.nodes[4].engine.peer_id();
    c.send(0, Event::HeartbeatTimeout { peer: silent });
    assert_eq!(c.nodes[0].engine.mode(), Mode::Searching);
    assert!(c.nodes[0].ended.is_empty(), "a drop does not end the entry");
    c.run(80);
    c.send(0, Event::UserStop);
    let f = &c.nodes[0].ended[0];
    assert_eq!(f.outcome, SearchOutcome::Stopped);
    // Later re-forms may time out unconfirmed, which counts as declines, not drops.
    assert_eq!(f.drops, 1);
    assert!(f.first_formed_at.is_some());
    assert!(f.elapsed > Duration::from_secs(60));
}

// ---- attempt counters (ADR 0026) --------------------------------------------------------

use super::diagnose::{self, Input, Situation};

fn diagnose_of(v: &View, peers: u32) -> Situation {
    let st = v.stats.expect("stats");
    let ended = v.mode == Mode::Expired;
    diagnose::diagnose(&Input {
        ended,
        elapsed: st.elapsed,
        network_peers: peers,
        seen: if ended { st.peak } else { st.current },
        own_role: Some(st.own_role),
        attempts: st.attempts,
    })
}

/// Node 0 is a tank; nodes 1 to 4 are a healer and three damage that go silent right after the
/// first ads, so every invitation the tank sends is unanswered.
fn silent_invitees(c: &mut Cluster) {
    five(c);
    for i in 1..5 {
        c.nodes[i].alive = false;
    }
}

#[test]
fn unanswered_invitations_give_nobody_answered_after_min_attempts() {
    let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
    silent_invitees(&mut c);
    c.run(7);
    let a = c.nodes[0].engine.view(c.now).stats.unwrap().attempts;
    assert_eq!(a.invites_sent, 4, "one round invites the four");
    assert!(a.invites_sent < diagnose::MIN_ATTEMPTS, "not yet enough");
    assert_eq!(a.accepts_got, 0);
    assert_eq!(a.invite_epochs, 1, "all four went out in one round");
    c.run(48);
    let v = c.nodes[0].engine.view(c.now);
    let a = v.stats.unwrap().attempts;
    assert!(a.invites_sent >= diagnose::MIN_ATTEMPTS);
    assert!(a.invite_epochs >= diagnose::MIN_INVITE_EPOCHS);
    assert_eq!((a.accepts_got, a.invites_got), (0, 0));
    assert_eq!(diagnose_of(&v, 4), Situation::NobodyAnswered);
}

#[test]
fn the_counters_are_frozen_at_expiry_and_reach_the_history_facts() {
    let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
    silent_invitees(&mut c);
    c.run(610);
    let v = c.nodes[0].engine.view(c.now);
    assert_eq!(v.mode, Mode::Expired);
    let a = v.stats.unwrap().attempts;
    assert!(
        a.invites_sent >= diagnose::MIN_ATTEMPTS,
        "not zeroed by stop()"
    );
    assert_eq!(diagnose_of(&v, 4), Situation::NobodyAnswered);
    let f = &c.nodes[0].ended[0];
    assert_eq!(f.outcome, SearchOutcome::Expired);
    assert_eq!((f.invites_sent, f.accepts_got), (a.invites_sent, 0));
    assert_eq!(f.taken_seen, 0);
}

#[test]
fn invitees_count_the_invitations_they_got_and_leaders_the_accepts() {
    let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
    five(&mut c);
    c.run(12);
    // The party formed (Committed): the stats still show the counters.
    let l = c.nodes[0].engine.view(c.now).stats.unwrap().attempts;
    assert!(l.invites_sent >= 4);
    assert!(l.accepts_got >= 4, "healer and three damage accepted");
    let got: u32 = (1..5)
        .map(|i| {
            c.nodes[i]
                .engine
                .view(c.now)
                .stats
                .unwrap()
                .attempts
                .invites_got
        })
        .sum();
    assert!(got >= 4);
}

#[test]
fn a_party_forming_next_to_a_searching_node_shows_others_grouping() {
    let mut c = Cluster::new(&[1, 2, 3, 4, 5, 6]);
    five(&mut c);
    // A Horde damage node sees the Alliance ads but can never join them.
    start(
        &mut c,
        5,
        profile(Role::Damage, 25, 100, Faction::Horde, "Horde"),
    );
    c.run(40);
    for i in 0..5 {
        assert_eq!(c.nodes[i].engine.mode(), Mode::Committed, "node {i}");
    }
    let v = c.nodes[5].engine.view(c.now);
    let st = v.stats.unwrap();
    assert_eq!(st.attempts.taken_seen, 5, "five distinct peers were taken");
    assert_eq!(st.current.players, 0);
    assert_eq!(diagnose_of(&v, 5), Situation::OthersGrouping);
    // No notice, no explanation: a lone node with no taken notice is `alone`.
    let mut lone = Cluster::new(&[1]);
    solo(&mut lone);
    lone.run(40);
    let v = lone.nodes[0].engine.view(lone.now);
    assert_eq!(v.stats.unwrap().attempts.taken_seen, 0);
    assert_eq!(diagnose_of(&v, 3), Situation::Alone);
}
