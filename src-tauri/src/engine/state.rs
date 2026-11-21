//! The engine state machine. It does no I/O and reads no clock: the host passes `now` (time
//! since the Unix epoch, so every peer derives the same epoch number) with each event and
//! carries out the returned actions.

use super::ad::{Ad, Faction, Message, PartyId, Role};
use super::party::{PARTY_SIZE, Party, build_parties, party_for, party_id};
use super::sample::Sample;
use super::widen;
use crate::config::{
    AD_CUTOFF, AD_KEEPALIVE, BAN, CONFIRM_WINDOW, EPOCH, READY_WINDOW, SAMPLE_SIZE,
};
use crate::dungeons::TopicId;
use crate::identity::PeerId;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

/// One topic being searched, with the dungeon's minimum level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicSearch {
    pub topic: TopicId,
    pub min_level: u8,
}

/// The player's own character as the engine needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub role: Role,
    pub level: u8,
    pub item_level: u16,
    pub faction: Faction,
    pub first_name: String,
    pub surname: String,
    pub zone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Tick,
    AdReceived {
        topic: TopicId,
        ad: Ad,
    },
    TakenReceived {
        topic: TopicId,
        peer: PeerId,
        counter: u16,
    },
    AnnounceReceived {
        topic: TopicId,
        from: PeerId,
        epoch: u64,
        members: [PeerId; PARTY_SIZE],
        party_id: PartyId,
    },
    ReadyReceived {
        topic: TopicId,
        from: PeerId,
        party_id: PartyId,
    },
    ConfirmReceived {
        topic: TopicId,
        from: PeerId,
        party_id: PartyId,
    },
    DeclineReceived {
        topic: TopicId,
        from: PeerId,
        party_id: PartyId,
    },
    HeartbeatReceived {
        topic: TopicId,
        from: PeerId,
        party_id: PartyId,
        nonce: u64,
    },
    HeartbeatAckReceived {
        from: PeerId,
        party_id: PartyId,
    },
    /// The host's heartbeat check found `peer` silent.
    HeartbeatTimeout {
        peer: PeerId,
    },
    UserConfirm,
    UserDecline,
    /// `topics` are searched from the start, `extra` are joined at the last widening stage.
    UserStart {
        profile: Profile,
        topics: Vec<TopicSearch>,
        extra: Vec<TopicSearch>,
    },
    UserStop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SendAd {
        topic: TopicId,
        ad: Ad,
    },
    /// `to: None` is a broadcast on the topic.
    SendMessage {
        topic: TopicId,
        to: Option<PeerId>,
        msg: Message,
    },
    Join(TopicId),
    Leave(TopicId),
    ShowState(View),
    /// Ban the peer locally. Never gossiped.
    BanLocal(PeerId, Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Idle,
    Searching,
    /// A party is committed and waiting for Ready and Confirm.
    Committed,
    /// All five confirmed.
    Final,
    /// The search limit passed.
    Expired,
}

/// What a member's ad claimed (self-reported, ADR 0003). Known only for members seen in a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberDetails {
    pub role: Role,
    pub level: u8,
    pub item_level: u16,
}

/// Why the last party fell apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropCause {
    /// `peer` clicked Decline (it may be this peer).
    Declined,
    /// Nobody was unreachable but the party did not finish in time.
    Timeout,
    /// `peer` did not answer and is banned locally.
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropInfo {
    pub cause: DropCause,
    pub peer: Option<PeerId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberView {
    pub peer: PeerId,
    pub details: Option<MemberDetails>,
    pub ready: bool,
    pub confirmed: bool,
    /// First name and surname, claimed by the member. Only shown once the party is final.
    pub name: Option<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub mode: Mode,
    pub stage: u8,
    /// Other searching peers currently in the samples.
    pub peers_seen: usize,
    pub next_stage_in: Option<Duration>,
    /// The Confirm prompt is showing and the player has not answered.
    pub confirm_prompt: bool,
    pub party: Vec<MemberView>,
    /// The topic and id of the party this peer is committed to (the host sends heartbeats on it).
    pub committed: Option<(TopicId, PartyId)>,
    /// Why the most recent party was dropped. Kept until the next commit.
    pub last_drop: Option<DropInfo>,
}

struct Search {
    profile: Profile,
    topics: Vec<TopicSearch>,
    extra: Vec<TopicSearch>,
    joined: BTreeSet<TopicId>,
    search_start: Duration,
    stage_start: Duration,
    stage: u8,
    last_ad: Duration,
    samples: BTreeMap<TopicId, Sample>,
    /// My latest announcement (topic, party, epoch). A peer announces at most one party per
    /// epoch, and only the latest one can be confirmed.
    announced: Option<(TopicId, PartyId)>,
    own_ad: Option<Ad>,
    last_epoch: Option<u64>,
    announces: HashMap<(TopicId, PartyId), Announce>,
}

struct Announce {
    epoch: u64,
    members: [PeerId; PARTY_SIZE],
    from: BTreeSet<PeerId>,
}

struct Commit {
    topic: TopicId,
    party: Party,
    since: Duration,
    ready: BTreeSet<PeerId>,
    alive: BTreeSet<PeerId>,
    confirmed: BTreeSet<PeerId>,
    names: BTreeMap<PeerId, (String, String)>,
    details: BTreeMap<PeerId, MemberDetails>,
    my_confirmed: bool,
}

pub struct Engine {
    me: PeerId,
    mode: Mode,
    bans: HashMap<PeerId, Duration>,
    counter: u16,
    search: Option<Search>,
    commit: Option<Commit>,
    /// Messages about a party that arrived before this peer committed to it.
    early: Vec<Event>,
    final_names: BTreeMap<PeerId, (String, String)>,
    final_party: Vec<PeerId>,
    final_details: BTreeMap<PeerId, MemberDetails>,
    last_drop: Option<DropInfo>,
}

const MAX_EARLY: usize = 64;

impl Engine {
    pub fn new(me: PeerId) -> Self {
        Self {
            me,
            mode: Mode::Idle,
            bans: HashMap::new(),
            counter: 0,
            search: None,
            commit: None,
            early: Vec::new(),
            final_names: BTreeMap::new(),
            final_party: Vec::new(),
            final_details: BTreeMap::new(),
            last_drop: None,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.me
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn is_banned(&self, peer: &PeerId, now: Duration) -> bool {
        self.bans.get(peer).is_some_and(|&until| now < until)
    }

    pub fn stage(&self) -> u8 {
        self.search.as_ref().map_or(0, |s| s.stage)
    }

    /// The committed or final party's members, if any.
    pub fn party_members(&self) -> Vec<PeerId> {
        match (&self.commit, self.mode) {
            (Some(c), _) => c.party.members.to_vec(),
            (None, Mode::Final) => self.final_party.clone(),
            _ => Vec::new(),
        }
    }

    pub fn handle(&mut self, now: Duration, event: Event) -> Vec<Action> {
        let mut out = Vec::new();
        self.dispatch(now, event, &mut out);
        out
    }

    fn dispatch(&mut self, now: Duration, event: Event, out: &mut Vec<Action>) {
        match event {
            Event::UserStart {
                profile,
                topics,
                extra,
            } => self.start(now, profile, topics, extra, out),
            Event::UserStop => self.stop(out),
            Event::Tick => self.tick(now, out),
            Event::AdReceived { topic, ad } => {
                self.on_ad(now, topic, ad, out);
            }
            Event::TakenReceived {
                topic,
                peer,
                counter,
            } => {
                if let Some(s) = self.search.as_mut()
                    && let Some(sample) = s.samples.get_mut(&topic)
                {
                    sample.mark_taken(peer, counter);
                }
            }
            Event::AnnounceReceived {
                topic,
                from,
                epoch,
                members,
                party_id: pid,
            } => self.on_announce(now, topic, from, epoch, members, pid, out),
            e @ (Event::ReadyReceived { .. }
            | Event::ConfirmReceived { .. }
            | Event::DeclineReceived { .. }) => self.on_party_message(now, e, out),
            Event::HeartbeatReceived {
                topic,
                from,
                party_id: pid,
                nonce,
            } => {
                if self
                    .commit
                    .as_ref()
                    .is_some_and(|c| c.party.id == pid && c.party.contains(&from))
                {
                    out.push(Action::SendMessage {
                        topic,
                        to: Some(from),
                        msg: Message::HeartbeatAck {
                            peer: self.me,
                            party_id: pid,
                            nonce,
                        },
                    });
                }
            }
            Event::HeartbeatAckReceived {
                from,
                party_id: pid,
            } => {
                if let Some(c) = self.commit.as_mut()
                    && c.party.id == pid
                    && c.party.contains(&from)
                {
                    c.alive.insert(from);
                }
            }
            Event::HeartbeatTimeout { peer } => {
                let in_party = self.commit.as_ref().is_some_and(|c| {
                    peer != self.me && c.party.contains(&peer) && !c.confirmed.contains(&peer)
                });
                if in_party {
                    self.drop_party(now, DropCause::Unreachable, Some(peer), out);
                }
            }
            Event::UserConfirm => self.user_confirm(now, out),
            Event::UserDecline => {
                if let Some(c) = &self.commit {
                    out.push(self.msg_to_all(
                        c.topic,
                        Message::Decline {
                            peer: self.me,
                            party_id: c.party.id,
                        },
                    ));
                    self.drop_party(now, DropCause::Declined, Some(self.me), out);
                }
            }
        }
    }

    fn msg_to_all(&self, topic: TopicId, msg: Message) -> Action {
        Action::SendMessage {
            topic,
            to: None,
            msg,
        }
    }

    // ---- search lifecycle -------------------------------------------------------------

    fn start(
        &mut self,
        now: Duration,
        profile: Profile,
        topics: Vec<TopicSearch>,
        extra: Vec<TopicSearch>,
        out: &mut Vec<Action>,
    ) {
        self.stop(out);
        let mut samples = BTreeMap::new();
        for t in topics.iter().chain(extra.iter()) {
            samples
                .entry(t.topic)
                .or_insert_with(|| Sample::new(self.me, profile.level, SAMPLE_SIZE));
        }
        let mut joined = BTreeSet::new();
        for t in &topics {
            if joined.insert(t.topic) {
                out.push(Action::Join(t.topic));
            }
        }
        self.search = Some(Search {
            profile,
            topics,
            extra,
            joined,
            search_start: now,
            stage_start: now,
            stage: 0,
            last_ad: now,
            samples,
            announced: None,
            own_ad: None,
            last_epoch: None,
            announces: HashMap::new(),
        });
        self.mode = Mode::Searching;
        self.send_ads(now, out);
        out.push(Action::ShowState(self.view(now)));
    }

    fn stop(&mut self, out: &mut Vec<Action>) {
        if let Some(s) = self.search.take() {
            for t in s.joined {
                out.push(Action::Leave(t));
            }
        }
        self.commit = None;
        self.early.clear();
        self.mode = Mode::Idle;
    }

    fn own_ad(&mut self, now: Duration) -> Ad {
        self.counter = self.counter.wrapping_add(1);
        let s = self.search.as_ref().expect("searching");
        Ad {
            peer: self.me,
            counter: self.counter,
            role: s.profile.role,
            secondary_role: None,
            level: s.profile.level,
            item_level: s.profile.item_level,
            faction: s.profile.faction,
            stage: s.stage,
            first_name: s.profile.first_name.clone(),
            surname: s.profile.surname.clone(),
            zone: s.profile.zone.clone(),
            created_at: now.as_secs() as u32,
        }
    }

    fn send_ads(&mut self, now: Duration, out: &mut Vec<Action>) {
        let ad = self.own_ad(now);
        let s = self.search.as_mut().expect("searching");
        s.last_ad = now;
        s.own_ad = Some(ad.clone());
        for topic in s.joined.iter().copied().collect::<Vec<_>>() {
            if let Some(sample) = s.samples.get_mut(&topic) {
                sample.insert(ad.clone(), now);
            }
            out.push(Action::SendAd {
                topic,
                ad: ad.clone(),
            });
        }
    }

    /// A peer we had not seen before: it just joined and has missed our ad (ads are only
    /// re-sent every keep-alive), so answer it directly, but only if we could be grouped.
    fn on_ad(&mut self, now: Duration, topic: TopicId, ad: Ad, out: &mut Vec<Action>) {
        if ad.peer == self.me {
            return;
        }
        let min_level = self.min_level(&topic);
        let Some(s) = self.search.as_mut() else {
            return;
        };
        let Some(sample) = s.samples.get_mut(&topic) else {
            return;
        };
        let is_new = sample.get(&ad.peer).is_none();
        let peer = ad.peer;
        let reply = is_new
            && sample.insert(ad.clone(), now)
            && s.joined.contains(&topic)
            && s.own_ad.as_ref().is_some_and(|own| {
                super::ad::compatible(own, &ad, own.stage.min(ad.stage), min_level)
            });
        if reply {
            let own = s.own_ad.clone().expect("checked");
            out.push(Action::SendMessage {
                topic,
                to: Some(peer),
                msg: Message::Ad(own),
            });
        }
    }

    fn min_level(&self, topic: &TopicId) -> u8 {
        self.search
            .as_ref()
            .and_then(|s| {
                s.topics
                    .iter()
                    .chain(s.extra.iter())
                    .find(|t| t.topic == *topic)
            })
            .map_or(1, |t| t.min_level)
    }

    // ---- time -------------------------------------------------------------------------

    fn tick(&mut self, now: Duration, out: &mut Vec<Action>) {
        self.bans.retain(|_, until| now < *until);
        match self.mode {
            Mode::Searching => self.tick_searching(now, out),
            Mode::Committed => self.tick_committed(now, out),
            _ => {}
        }
    }

    fn tick_searching(&mut self, now: Duration, out: &mut Vec<Action>) {
        let (expired, stage_now) = {
            let s = self.search.as_ref().expect("searching");
            (
                widen::search_expired(now.saturating_sub(s.search_start)),
                widen::stage_at(now.saturating_sub(s.stage_start)),
            )
        };
        if expired {
            self.stop(out);
            self.mode = Mode::Expired;
            out.push(Action::ShowState(self.view(now)));
            return;
        }
        let mut changed = false;
        {
            let s = self.search.as_mut().expect("searching");
            if stage_now != s.stage {
                s.stage = stage_now;
                changed = true;
                if widen::rules(stage_now).other_dungeons {
                    let extra = s.extra.clone();
                    for t in extra {
                        if s.joined.insert(t.topic) {
                            out.push(Action::Join(t.topic));
                        }
                    }
                }
            }
        }
        let keepalive_due = {
            let s = self.search.as_ref().expect("searching");
            now.saturating_sub(s.last_ad) >= AD_KEEPALIVE
        };
        if changed || keepalive_due {
            self.send_ads(now, out);
        }
        if changed {
            out.push(Action::ShowState(self.view(now)));
        }
        let epoch = now.as_secs() / EPOCH.as_secs();
        let due = self
            .search
            .as_ref()
            .expect("searching")
            .last_epoch
            .is_none_or(|l| l < epoch);
        if due {
            self.run_epoch(now, epoch, out);
        }
    }

    fn run_epoch(&mut self, now: Duration, epoch: u64, out: &mut Vec<Action>) {
        let epoch_start = Duration::from_secs(epoch * EPOCH.as_secs());
        let cutoff = epoch_start.saturating_sub(AD_CUTOFF);
        let me = self.me;
        let bans = self.bans.clone();
        let mut found: Option<(TopicId, Party)> = None;
        {
            let s = self.search.as_mut().expect("searching");
            s.last_epoch = Some(epoch);
            s.announces.retain(|_, a| a.epoch + 1 >= epoch);
            for sample in s.samples.values_mut() {
                sample.expire(now);
            }
        }
        let topics: Vec<TopicId> = self
            .search
            .as_ref()
            .unwrap()
            .joined
            .iter()
            .copied()
            .collect();
        for topic in topics {
            let min_level = self.min_level(&topic);
            let s = self.search.as_ref().unwrap();
            let Some(sample) = s.samples.get(&topic) else {
                continue;
            };
            let cands: Vec<&Ad> = sample
                .eligible(now, cutoff)
                .into_iter()
                .filter(|a| a.peer == me || !bans.get(&a.peer).is_some_and(|&u| now < u))
                .collect();
            let parties = build_parties(&topic, epoch, &cands, min_level);
            if let Some(p) = party_for(&parties, &me) {
                found = Some((topic, p));
                break;
            }
        }
        let Some((topic, party)) = found else { return };
        let s = self.search.as_mut().unwrap();
        s.announced = Some((topic, party.id));
        let entry = s.announces.entry((topic, party.id)).or_insert(Announce {
            epoch,
            members: party.members,
            from: BTreeSet::new(),
        });
        entry.from.insert(me);
        out.push(Action::SendMessage {
            topic,
            to: None,
            msg: Message::PartyAnnounce {
                peer: me,
                epoch,
                members: party.members,
                party_id: party.id,
            },
        });
        self.check_commit(now, topic, party.id, out);
    }

    // ---- agreeing on a party ----------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn on_announce(
        &mut self,
        now: Duration,
        topic: TopicId,
        from: PeerId,
        epoch: u64,
        members: [PeerId; PARTY_SIZE],
        pid: PartyId,
        out: &mut Vec<Action>,
    ) {
        if self.mode != Mode::Searching {
            return;
        }
        let cur = now.as_secs() / EPOCH.as_secs();
        // An announcement for the previous or next epoch is kept for one epoch.
        if epoch + 1 < cur || epoch > cur + 1 {
            return;
        }
        let mut sorted = members;
        sorted.sort();
        if sorted != members || sorted.windows(2).any(|w| w[0] == w[1]) {
            return;
        }
        if !members.contains(&from) || party_id(&topic, epoch, &members) != pid {
            return;
        }
        let s = self.search.as_mut().unwrap();
        if !s.joined.contains(&topic) {
            return;
        }
        s.announces
            .entry((topic, pid))
            .or_insert(Announce {
                epoch,
                members,
                from: BTreeSet::new(),
            })
            .from
            .insert(from);
        self.check_commit(now, topic, pid, out);
    }

    fn check_commit(&mut self, now: Duration, topic: TopicId, pid: PartyId, out: &mut Vec<Action>) {
        if self.mode != Mode::Searching {
            return;
        }
        let s = self.search.as_ref().unwrap();
        if s.announced != Some((topic, pid)) {
            return; // I moved on to another party (or have none): I cannot confirm this one.
        }
        let Some(a) = s.announces.get(&(topic, pid)) else {
            return;
        };
        if !a.members.iter().all(|m| a.from.contains(m)) {
            return;
        }
        let party = Party {
            id: pid,
            epoch: a.epoch,
            members: a.members,
        };
        self.commit_to(now, topic, party, out);
    }

    fn commit_to(&mut self, now: Duration, topic: TopicId, party: Party, out: &mut Vec<Action>) {
        let me = self.me;
        let mut names = BTreeMap::new();
        let mut details = BTreeMap::new();
        {
            let s = self.search.as_mut().unwrap();
            for m in party.members {
                if let Some(e) = s.samples.get(&topic).and_then(|sm| sm.get(&m)) {
                    names.insert(m, (e.ad.first_name.clone(), e.ad.surname.clone()));
                    details.insert(m, details_of(&e.ad));
                }
            }
            if let Some(e) = s.samples.get(&topic).and_then(|sm| sm.get(&me)) {
                names.insert(me, (e.ad.first_name.clone(), e.ad.surname.clone()));
                details.insert(me, details_of(&e.ad));
            }
            // Members are taken: drop them from every sample. Stop announcing.
            for sample in s.samples.values_mut() {
                for m in party.members {
                    if m != me {
                        sample.remove(&m);
                    }
                }
            }
            s.announces.clear();
            // Leave the other topics. Rejoined if the party is dropped.
            for t in s.joined.iter().copied().collect::<Vec<_>>() {
                if t != topic {
                    s.joined.remove(&t);
                    out.push(Action::Leave(t));
                }
            }
        }
        out.push(self.msg_to_all(
            topic,
            Message::Taken {
                peer: me,
                counter: self.counter,
            },
        ));
        out.push(self.msg_to_all(
            topic,
            Message::Ready {
                peer: me,
                party_id: party.id,
                nonce: 0,
            },
        ));
        self.mode = Mode::Committed;
        let mut ready = BTreeSet::new();
        ready.insert(me);
        let mut alive = BTreeSet::new();
        alive.insert(me);
        self.commit = Some(Commit {
            topic,
            party,
            since: now,
            ready,
            alive,
            confirmed: BTreeSet::new(),
            names,
            details,
            my_confirmed: false,
        });
        self.last_drop = None;
        // Replay messages that arrived before the commit.
        for e in std::mem::take(&mut self.early) {
            if self.mode == Mode::Committed {
                self.on_party_message(now, e, out);
            }
        }
        if self.mode == Mode::Committed {
            out.push(Action::ShowState(self.view(now)));
        }
    }

    fn on_party_message(&mut self, now: Duration, ev: Event, out: &mut Vec<Action>) {
        let (from, pid, kind) = match &ev {
            Event::ReadyReceived { from, party_id, .. } => (*from, *party_id, 0),
            Event::ConfirmReceived { from, party_id, .. } => (*from, *party_id, 1),
            Event::DeclineReceived { from, party_id, .. } => (*from, *party_id, 2),
            _ => return,
        };
        let Some(c) = self.commit.as_mut() else {
            if self.mode == Mode::Searching && self.early.len() < MAX_EARLY {
                self.early.push(ev);
            }
            return;
        };
        if c.party.id != pid {
            if self.mode == Mode::Committed && self.early.len() < MAX_EARLY {
                self.early.push(ev);
            }
            return;
        }
        if !c.party.contains(&from) {
            return;
        }
        match kind {
            0 => {
                c.ready.insert(from);
                c.alive.insert(from);
            }
            1 => {
                c.confirmed.insert(from);
                c.ready.insert(from);
                c.alive.insert(from);
            }
            _ => {
                self.drop_party(now, DropCause::Declined, Some(from), out);
                return;
            }
        }
        self.check_final(now, out);
        if self.mode == Mode::Committed {
            out.push(Action::ShowState(self.view(now)));
        }
    }

    fn user_confirm(&mut self, now: Duration, out: &mut Vec<Action>) {
        let me = self.me;
        let Some(c) = self.commit.as_mut() else {
            return;
        };
        if c.my_confirmed {
            return;
        }
        c.my_confirmed = true;
        c.confirmed.insert(me);
        let (topic, pid) = (c.topic, c.party.id);
        out.push(self.msg_to_all(
            topic,
            Message::Confirm {
                peer: me,
                party_id: pid,
            },
        ));
        self.check_final(now, out);
        if self.mode == Mode::Committed {
            out.push(Action::ShowState(self.view(now)));
        }
    }

    fn check_final(&mut self, now: Duration, out: &mut Vec<Action>) {
        let done = self
            .commit
            .as_ref()
            .is_some_and(|c| c.party.members.iter().all(|m| c.confirmed.contains(m)));
        if !done {
            return;
        }
        let c = self.commit.take().unwrap();
        self.final_names = c.names;
        self.final_details = c.details;
        self.final_party = c.party.members.to_vec();
        if let Some(s) = self.search.take() {
            for t in s.joined {
                out.push(Action::Leave(t));
            }
        }
        self.mode = Mode::Final;
        out.push(Action::ShowState(self.view(now)));
    }

    fn tick_committed(&mut self, now: Duration, out: &mut Vec<Action>) {
        let c = self.commit.as_ref().expect("committed");
        let age = now.saturating_sub(c.since);
        if age >= READY_WINDOW {
            let silent = c
                .party
                .members
                .iter()
                .copied()
                .find(|m| !c.alive.contains(m));
            if let Some(m) = silent {
                self.drop_party(now, DropCause::Unreachable, Some(m), out);
                return;
            }
        }
        if age >= CONFIRM_WINDOW && !c.my_confirmed {
            let (topic, pid) = (c.topic, c.party.id);
            out.push(self.msg_to_all(
                topic,
                Message::Decline {
                    peer: self.me,
                    party_id: pid,
                },
            ));
            self.drop_party(now, DropCause::Timeout, None, out);
            return;
        }
        // Someone else never answered the human prompt and never sent Decline (crashed after
        // Ready): the party times out without a penalty.
        if age >= CONFIRM_WINDOW + READY_WINDOW {
            self.drop_party(now, DropCause::Timeout, None, out);
        }
    }

    /// Back to searching. `ban` is set when a member did not answer: a local ban, not sent.
    fn drop_party(
        &mut self,
        now: Duration,
        cause: DropCause,
        who: Option<PeerId>,
        out: &mut Vec<Action>,
    ) {
        let ban = if cause == DropCause::Unreachable {
            who
        } else {
            None
        };
        self.last_drop = Some(DropInfo { cause, peer: who });
        self.commit = None;
        self.early.clear();
        if let Some(p) = ban {
            self.bans.insert(p, now + BAN);
            out.push(Action::BanLocal(p, BAN));
        }
        self.mode = Mode::Searching;
        if let Some(s) = self.search.as_mut() {
            s.stage_start = now;
            s.stage = 0;
            s.announced = None;
            s.announces.clear();
            s.last_epoch = Some(now.as_secs() / EPOCH.as_secs());
            // Stage 0 again: leave the extra topics, rejoin the primary ones.
            let primary: BTreeSet<TopicId> = s.topics.iter().map(|t| t.topic).collect();
            for t in s.joined.clone() {
                if !primary.contains(&t) {
                    s.joined.remove(&t);
                    out.push(Action::Leave(t));
                }
            }
            for t in &primary {
                if s.joined.insert(*t) {
                    out.push(Action::Join(*t));
                }
            }
        }
        self.send_ads(now, out);
        out.push(Action::ShowState(self.view(now)));
    }

    // ---- view -------------------------------------------------------------------------

    pub fn view(&self, now: Duration) -> View {
        let mut v = View {
            mode: self.mode,
            stage: 0,
            peers_seen: 0,
            next_stage_in: None,
            confirm_prompt: false,
            party: Vec::new(),
            committed: None,
            last_drop: self.last_drop,
        };
        if let Some(s) = &self.search {
            v.stage = s.stage;
            let mut seen = BTreeSet::new();
            for sample in s.samples.values() {
                for p in sample_peers(sample) {
                    if p != self.me {
                        seen.insert(p);
                    }
                }
            }
            v.peers_seen = seen.len();
            v.next_stage_in = widen::until_next_stage(now.saturating_sub(s.stage_start));
        }
        if let Some(c) = &self.commit {
            v.confirm_prompt = !c.my_confirmed;
            v.committed = Some((c.topic, c.party.id));
            v.party = c
                .party
                .members
                .iter()
                .map(|m| MemberView {
                    peer: *m,
                    ready: c.ready.contains(m),
                    confirmed: c.confirmed.contains(m),
                    details: c.details.get(m).copied(),
                    name: None,
                })
                .collect();
        }
        if self.mode == Mode::Final {
            v.party = self
                .final_party
                .iter()
                .map(|m| MemberView {
                    peer: *m,
                    ready: true,
                    confirmed: true,
                    details: self.final_details.get(m).copied(),
                    name: self.final_names.get(m).cloned(),
                })
                .collect();
        }
        v
    }
}

fn details_of(ad: &Ad) -> MemberDetails {
    MemberDetails {
        role: ad.role,
        level: ad.level,
        item_level: ad.item_level,
    }
}

fn sample_peers(sample: &Sample) -> Vec<PeerId> {
    sample
        .eligible(Duration::ZERO, Duration::MAX)
        .iter()
        .map(|a| a.peer)
        .collect()
}
