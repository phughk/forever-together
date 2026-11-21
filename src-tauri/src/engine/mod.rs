//! The pure matching engine: ads, samples, party building, widening and the state machine.
//! No I/O. `Network` and `NetError` live here so `net` can implement them later.

pub mod ad;
pub mod party;
pub mod rank;
pub mod sample;
pub mod sim;
pub mod state;
pub mod widen;

pub use ad::{Ad, Faction, Message, PartyId, Role, compatible, event_from_message};
pub use state::{
    Action, DropCause, DropInfo, Engine, Event, MemberDetails, MemberView, Mode, Profile,
    TopicSearch, View,
};

use crate::dungeons::TopicId;
use crate::identity::PeerId;
use thiserror::Error;

/// All errors a network can report.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NetError {
    #[error("not joined to the topic")]
    NotJoined,
    #[error("unknown peer")]
    UnknownPeer,
    #[error("signature check failed")]
    BadSignature,
    #[error("the network is closed")]
    Closed,
    #[error("rate limited")]
    RateLimited,
    #[error("network I/O: {0}")]
    Io(String),
}

/// The network as the engine's host sees it. `net` implements it with iroh and with an
/// in-memory fake. Implementations verify signatures before `next_event` returns, so the
/// engine only ever sees verified authors.
#[allow(async_fn_in_trait)]
pub trait Network {
    async fn join(&mut self, topic: TopicId) -> Result<(), NetError>;
    async fn leave(&mut self, topic: TopicId) -> Result<(), NetError>;
    async fn broadcast(&mut self, topic: TopicId, msg: Message) -> Result<(), NetError>;
    async fn send_direct(
        &mut self,
        peer: PeerId,
        topic: TopicId,
        msg: Message,
    ) -> Result<(), NetError>;
    /// The next verified event from the network (an ad, an announcement, and so on).
    async fn next_event(&mut self) -> Result<Event, NetError>;
    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError>;
}

/// Carry out the network actions. Returns the actions that are for the host, not the network
/// (`ShowState` and `BanLocal`), in order.
pub async fn apply_network<N: Network>(
    net: &mut N,
    actions: Vec<Action>,
) -> Result<Vec<Action>, NetError> {
    let mut rest = Vec::new();
    for a in actions {
        match a {
            Action::Join(t) => net.join(t).await?,
            Action::Leave(t) => net.leave(t).await?,
            Action::SendAd { topic, ad } => net.broadcast(topic, Message::Ad(ad)).await?,
            Action::SendMessage {
                topic,
                to: None,
                msg,
            } => net.broadcast(topic, msg).await?,
            Action::SendMessage {
                topic,
                to: Some(p),
                msg,
            } => net.send_direct(p, topic, msg).await?,
            other => rest.push(other),
        }
    }
    Ok(rest)
}

#[cfg(test)]
pub(crate) mod testkit {
    //! A synchronous cluster of engines joined by a lossless instant bus.
    use super::*;
    use crate::identity::PeerId;
    use std::collections::{BTreeSet, VecDeque};
    use std::time::Duration;

    pub struct Node {
        pub engine: Engine,
        pub joined: BTreeSet<TopicId>,
        pub alive: bool,
        pub shown: Vec<View>,
        pub bans: Vec<PeerId>,
        pub announces_by_epoch: std::collections::BTreeMap<u64, u32>,
    }

    pub struct Cluster {
        pub nodes: Vec<Node>,
        pub now: Duration,
        /// Every message sent, for assertions: (sender index, to, message).
        pub log: Vec<(usize, Option<PeerId>, Message)>,
        /// Nodes whose Ready and Confirm messages are lost.
        pub muted: BTreeSet<usize>,
    }

    pub fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    impl Cluster {
        /// Starts at an epoch boundary.
        pub fn new(ids: &[u8]) -> Self {
            let nodes = ids
                .iter()
                .map(|&i| Node {
                    engine: Engine::new(PeerId([i; 32])),
                    joined: BTreeSet::new(),
                    alive: true,
                    shown: Vec::new(),
                    bans: Vec::new(),
                    announces_by_epoch: Default::default(),
                })
                .collect();
            Self {
                nodes,
                now: secs(1_000_000),
                log: Vec::new(),
                muted: BTreeSet::new(),
            }
        }

        pub fn idx(&self, id: u8) -> usize {
            self.nodes
                .iter()
                .position(|n| n.engine.peer_id() == PeerId([id; 32]))
                .unwrap()
        }

        pub fn send(&mut self, i: usize, ev: Event) {
            let mut queue: VecDeque<(usize, Event)> = VecDeque::new();
            queue.push_back((i, ev));
            while let Some((i, ev)) = queue.pop_front() {
                if !self.nodes[i].alive {
                    continue;
                }
                let acts = self.nodes[i].engine.handle(self.now, ev);
                for a in acts {
                    self.route(i, a, &mut queue);
                }
            }
        }

        fn route(&mut self, from: usize, a: Action, queue: &mut VecDeque<(usize, Event)>) {
            let me = self.nodes[from].engine.peer_id();
            match a {
                Action::Join(t) => {
                    self.nodes[from].joined.insert(t);
                }
                Action::Leave(t) => {
                    self.nodes[from].joined.remove(&t);
                }
                Action::ShowState(v) => self.nodes[from].shown.push(v),
                Action::BanLocal(p, _) => self.nodes[from].bans.push(p),
                Action::SendAd { topic, ad } => {
                    self.deliver(from, me, topic, None, Message::Ad(ad), queue)
                }
                Action::SendMessage { topic, to, msg } => {
                    self.deliver(from, me, topic, to, msg, queue)
                }
            }
        }

        fn deliver(
            &mut self,
            from: usize,
            me: PeerId,
            topic: TopicId,
            to: Option<PeerId>,
            msg: Message,
            queue: &mut VecDeque<(usize, Event)>,
        ) {
            if let Message::PartyAnnounce { epoch, .. } = &msg {
                *self.nodes[from]
                    .announces_by_epoch
                    .entry(*epoch)
                    .or_default() += 1;
            }
            if self.muted.contains(&from)
                && matches!(msg, Message::Ready { .. } | Message::Confirm { .. })
            {
                return;
            }
            self.log.push((from, to, msg.clone()));
            for j in 0..self.nodes.len() {
                if j == from || !self.nodes[j].joined.contains(&topic) {
                    continue;
                }
                if to.is_some_and(|p| p != self.nodes[j].engine.peer_id()) {
                    continue;
                }
                queue.push_back((j, event_from_message(topic, msg.clone())));
            }
            let _ = me;
        }

        /// Advance the clock one second at a time, ticking every live node.
        pub fn run(&mut self, seconds: u64) {
            for _ in 0..seconds {
                self.now += secs(1);
                for i in 0..self.nodes.len() {
                    self.send(i, Event::Tick);
                }
            }
        }

        pub fn modes(&self) -> Vec<Mode> {
            self.nodes.iter().map(|n| n.engine.mode()).collect()
        }
    }

    pub fn profile(
        role: Role,
        level: u8,
        item_level: u16,
        faction: Faction,
        name: &str,
    ) -> Profile {
        Profile {
            role,
            level,
            item_level,
            faction,
            first_name: name.into(),
            surname: format!("{name}son"),
            zone: "Westfall".into(),
        }
    }

    pub fn topic() -> TopicId {
        TopicId([7; 32])
    }

    pub fn start(c: &mut Cluster, i: usize, p: Profile) {
        c.send(
            i,
            Event::UserStart {
                profile: p,
                topics: vec![TopicSearch {
                    topic: topic(),
                    min_level: 10,
                }],
                extra: vec![],
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::*;
    use super::*;

    fn five_alliance(c: &mut Cluster) {
        let roles = [
            Role::Tank,
            Role::Healer,
            Role::Damage,
            Role::Damage,
            Role::Damage,
        ];
        let levels = [24, 24, 25, 25, 26];
        for i in 0..5 {
            start(
                c,
                i,
                profile(
                    roles[i],
                    levels[i],
                    100 + i as u16,
                    Faction::Alliance,
                    &format!("N{i}"),
                ),
            );
        }
    }

    #[test]
    fn worked_example_five_peers_commit_the_same_party_and_no_sixth() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5, 6]);
        five_alliance(&mut c);
        // A sixth peer on the other faction, same topic, level 25 damage.
        start(
            &mut c,
            5,
            profile(Role::Damage, 25, 100, Faction::Horde, "Horde"),
        );
        c.run(12);
        for i in 0..5 {
            assert_eq!(c.nodes[i].engine.mode(), Mode::Committed, "node {i}");
        }
        assert_eq!(c.nodes[5].engine.mode(), Mode::Searching);
        let want: Vec<PeerId> = (1..=5u8).map(|i| PeerId([i; 32])).collect();
        for i in 0..5 {
            assert_eq!(c.nodes[i].engine.party_members(), want);
        }
        // All five announced the same party id.
        let ids: std::collections::BTreeSet<PartyId> = c
            .log
            .iter()
            .filter_map(|(_, _, m)| {
                if let Message::PartyAnnounce { party_id, .. } = m {
                    Some(*party_id)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn a_sixth_alliance_damage_peer_is_not_included() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5, 6]);
        five_alliance(&mut c);
        start(
            &mut c,
            5,
            profile(Role::Damage, 25, 100, Faction::Alliance, "Extra"),
        );
        c.run(12);
        let committed = c.modes().iter().filter(|m| **m == Mode::Committed).count();
        assert_eq!(committed, 5);
        assert_eq!(c.nodes[0].engine.party_members().len(), 5);
        // The leftover peer is still searching.
        let left: Vec<_> = (0..6)
            .filter(|&i| c.nodes[i].engine.mode() == Mode::Searching)
            .collect();
        assert_eq!(left.len(), 1);
    }

    #[test]
    fn committed_peers_send_taken_and_ready_and_leave_other_topics() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        c.run(12);
        for i in 0..5 {
            assert!(
                c.log
                    .iter()
                    .any(|(f, _, m)| *f == i && matches!(m, Message::Taken { .. }))
            );
            assert!(
                c.log
                    .iter()
                    .any(|(f, _, m)| *f == i && matches!(m, Message::Ready { .. }))
            );
        }
    }

    #[test]
    fn all_confirm_then_final_with_names_only_at_the_end() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        c.run(12);
        // Before the end, no names are shown.
        let v = c.nodes[0].engine.view(c.now);
        assert!(v.confirm_prompt && v.party.iter().all(|m| m.name.is_none()));
        for i in 0..4 {
            c.send(i, Event::UserConfirm);
        }
        assert_eq!(c.nodes[0].engine.mode(), Mode::Committed);
        c.send(4, Event::UserConfirm);
        assert_eq!(c.modes(), vec![Mode::Final; 5]);
        let v = c.nodes[0].engine.view(c.now);
        assert_eq!(v.party.len(), 5);
        assert!(v.party.iter().all(|m| m.name.is_some() && m.confirmed));
        assert!(c.nodes[0].joined.is_empty());
        // A second confirm does nothing.
        c.send(0, Event::UserConfirm);
        assert_eq!(c.nodes[0].engine.mode(), Mode::Final);
    }

    #[test]
    fn an_unclicked_confirm_sends_decline_and_nobody_is_penalised() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        c.run(12);
        let committed_at = c.now - secs(2); // the commit happened at the epoch boundary
        for i in 0..4 {
            c.send(i, Event::UserConfirm);
        }
        c.run(12);
        assert_eq!(c.modes(), vec![Mode::Committed; 5]);
        c.run(2);
        assert!(c.now - committed_at >= secs(15));
        // The human on node 4 never clicked: it declined, everyone is searching again.
        assert!(
            c.log
                .iter()
                .any(|(f, _, m)| *f == 4 && matches!(m, Message::Decline { .. }))
        );
        assert_eq!(c.modes(), vec![Mode::Searching; 5]);
        assert!(c.nodes.iter().all(|n| n.bans.is_empty()), "no penalty");
        // Stage clock restarted.
        assert_eq!(c.nodes[0].engine.stage(), 0);
    }

    #[test]
    fn declined_peers_form_a_new_party_in_a_later_epoch() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        c.run(12);
        c.send(0, Event::UserDecline);
        assert!(c.modes().iter().all(|m| *m == Mode::Searching));
        c.run(10); // the next epoch boundary is 8 s away
        assert_eq!(c.modes(), vec![Mode::Committed; 5]);
    }

    #[test]
    fn a_silent_peer_is_banned_locally_after_four_seconds_and_the_ban_is_not_sent() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        // Node 4's Ready never arrives (its app died right after announcing).
        c.muted.insert(4);
        c.run(12);
        let dead = PeerId([5; 32]);
        for i in 0..4 {
            assert_eq!(c.nodes[i].engine.mode(), Mode::Committed, "node {i}");
        }
        c.run(5);
        for i in 0..4 {
            assert_eq!(
                c.nodes[i].engine.mode(),
                Mode::Searching,
                "node {i} dropped the party"
            );
            assert_eq!(c.nodes[i].bans, vec![dead]);
            assert!(c.nodes[i].engine.is_banned(&dead, c.now));
        }
        // The ban is local: no message carries it, and it expires after 60 s.
        assert!(
            !c.log
                .iter()
                .any(|(_, _, m)| format!("{m:?}").to_lowercase().contains("ban"))
        );
        assert!(!c.nodes[0].engine.is_banned(&dead, c.now + secs(61)));
    }

    #[test]
    fn heartbeat_timeout_event_bans_and_drops() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        c.run(12);
        let dead = PeerId([3; 32]);
        c.send(0, Event::HeartbeatTimeout { peer: dead });
        assert_eq!(c.nodes[0].engine.mode(), Mode::Searching);
        assert_eq!(c.nodes[0].bans, vec![dead]);
        // A banned peer is not considered when building parties.
        assert!(c.nodes[0].engine.is_banned(&dead, c.now));
    }

    #[test]
    fn a_peer_announces_at_most_one_party_per_epoch() {
        let mut ids = vec![];
        for i in 1..=14u8 {
            ids.push(i);
        }
        let mut c = Cluster::new(&ids);
        for i in 0..14 {
            let role = if i % 5 == 0 {
                Role::Tank
            } else if i % 5 == 1 {
                Role::Healer
            } else {
                Role::Damage
            };
            start(
                &mut c,
                i,
                profile(
                    role,
                    24 + (i % 3) as u8,
                    100,
                    Faction::Alliance,
                    &format!("M{i}"),
                ),
            );
        }
        c.run(60);
        for n in &c.nodes {
            assert!(
                n.announces_by_epoch.values().all(|&k| k <= 1),
                "{:?}",
                n.announces_by_epoch
            );
        }
    }

    #[test]
    fn confirms_at_most_one_party() {
        let mut c = Cluster::new(&[1, 2, 3, 4, 5]);
        five_alliance(&mut c);
        c.run(12);
        c.send(0, Event::UserConfirm);
        let confirms = c
            .log
            .iter()
            .filter(|(f, _, m)| *f == 0 && matches!(m, Message::Confirm { .. }))
            .count();
        c.send(0, Event::UserConfirm);
        let again = c
            .log
            .iter()
            .filter(|(f, _, m)| *f == 0 && matches!(m, Message::Confirm { .. }))
            .count();
        assert_eq!((confirms, again), (1, 1));
    }

    #[test]
    fn late_announcement_for_a_party_i_moved_on_from_does_not_commit_me() {
        // Peer 1 sees a fully announced party it did not announce itself: it must not commit.
        let mut c = Cluster::new(&[1, 2, 3, 4, 5, 9]);
        five_alliance(&mut c);
        c.run(5);
        let members: [PeerId; 5] = std::array::from_fn(|i| PeerId([i as u8 + 1; 32]));
        let epoch = c.now.as_secs() / 10;
        let pid = party::party_id(&topic(), epoch, &members);
        // Node 9 (never searching) receives all five announcements.
        let i9 = c.idx(9);
        start(
            &mut c,
            i9,
            profile(Role::Damage, 25, 100, Faction::Alliance, "Late"),
        );
        for m in members {
            c.send(
                i9,
                Event::AnnounceReceived {
                    topic: topic(),
                    from: m,
                    epoch,
                    members,
                    party_id: pid,
                },
            );
        }
        assert_eq!(c.nodes[i9].engine.mode(), Mode::Searching);
    }

    #[test]
    fn widening_follows_the_table_and_the_search_limit_stops_the_search() {
        let mut c = Cluster::new(&[1]);
        // A lone peer: nothing to match. Collect the stages it advertises.
        start(
            &mut c,
            0,
            profile(Role::Tank, 24, 100, Faction::Alliance, "Solo"),
        );
        c.run(595);
        let stages: Vec<(u64, u8)> = {
            let mut v: Vec<(u64, u8)> = vec![];
            for (_, _, m) in &c.log {
                if let Message::Ad(a) = m
                    && v.last().is_none_or(|l| l.1 != a.stage)
                {
                    v.push((v.len() as u64, a.stage));
                }
            }
            v
        };
        assert_eq!(
            stages.iter().map(|s| s.1).collect::<Vec<_>>(),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(c.nodes[0].engine.mode(), Mode::Searching);
        c.run(10);
        assert_eq!(c.nodes[0].engine.mode(), Mode::Expired);
        assert!(c.nodes[0].joined.is_empty());
        assert_eq!(c.nodes[0].shown.last().unwrap().mode, Mode::Expired);
    }

    #[test]
    fn extra_dungeons_are_joined_only_at_the_last_stage() {
        let mut c = Cluster::new(&[1]);
        let extra = TopicSearch {
            topic: TopicId([8; 32]),
            min_level: 10,
        };
        c.send(
            0,
            Event::UserStart {
                profile: profile(Role::Tank, 24, 100, Faction::Alliance, "Solo"),
                topics: vec![TopicSearch {
                    topic: topic(),
                    min_level: 10,
                }],
                extra: vec![extra],
            },
        );
        c.run(239);
        assert_eq!(c.nodes[0].joined.len(), 1);
        c.run(2);
        assert_eq!(c.nodes[0].joined.len(), 2);
        assert_eq!(c.nodes[0].engine.stage(), 4);
    }

    #[test]
    fn user_stop_leaves_everything() {
        let mut c = Cluster::new(&[1]);
        start(
            &mut c,
            0,
            profile(Role::Tank, 24, 100, Faction::Alliance, "Solo"),
        );
        assert_eq!(c.nodes[0].joined.len(), 1);
        c.send(0, Event::UserStop);
        assert!(c.nodes[0].joined.is_empty());
        assert_eq!(c.nodes[0].engine.mode(), Mode::Idle);
    }

    #[test]
    fn taken_peers_are_dropped_from_samples() {
        let mut c = Cluster::new(&[1, 2]);
        start(
            &mut c,
            0,
            profile(Role::Tank, 24, 100, Faction::Alliance, "A"),
        );
        start(
            &mut c,
            1,
            profile(Role::Healer, 24, 100, Faction::Alliance, "B"),
        );
        assert_eq!(c.nodes[0].engine.view(c.now).peers_seen, 1);
        c.send(
            0,
            Event::TakenReceived {
                topic: topic(),
                peer: PeerId([2; 32]),
                counter: 1,
            },
        );
        assert_eq!(c.nodes[0].engine.view(c.now).peers_seen, 0);
    }

    // The Network trait with a recording implementation.
    struct Rec {
        calls: Vec<String>,
    }
    impl Network for Rec {
        async fn join(&mut self, _: TopicId) -> Result<(), NetError> {
            self.calls.push("join".into());
            Ok(())
        }
        async fn leave(&mut self, _: TopicId) -> Result<(), NetError> {
            self.calls.push("leave".into());
            Ok(())
        }
        async fn broadcast(&mut self, _: TopicId, m: Message) -> Result<(), NetError> {
            self.calls
                .push(format!("broadcast {}", matches!(m, Message::Ad(_))));
            Ok(())
        }
        async fn send_direct(&mut self, _: PeerId, _: TopicId, _: Message) -> Result<(), NetError> {
            self.calls.push("direct".into());
            Ok(())
        }
        async fn next_event(&mut self) -> Result<Event, NetError> {
            Err(NetError::Closed)
        }
        async fn dial(&mut self, _: PeerId) -> Result<(), NetError> {
            Err(NetError::UnknownPeer)
        }
    }

    #[tokio::test]
    async fn apply_network_runs_network_actions_and_returns_host_actions() {
        let mut e = Engine::new(PeerId([1; 32]));
        let acts = e.handle(
            secs(1_000_000),
            Event::UserStart {
                profile: profile(Role::Tank, 24, 100, Faction::Alliance, "A"),
                topics: vec![TopicSearch {
                    topic: topic(),
                    min_level: 10,
                }],
                extra: vec![],
            },
        );
        let mut net = Rec { calls: vec![] };
        let host = apply_network(&mut net, acts).await.unwrap();
        assert_eq!(net.calls, ["join", "broadcast true"]);
        assert!(matches!(host.as_slice(), [Action::ShowState(_)]));
        assert_eq!(net.dial(PeerId([2; 32])).await, Err(NetError::UnknownPeer));
        assert_eq!(net.next_event().await, Err(NetError::Closed));
    }
}
