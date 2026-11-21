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
