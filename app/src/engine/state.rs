//! The engine state machine. It does no I/O and reads no clock: the host passes `now` (time
//! since the Unix epoch, so every peer derives the same epoch number) with each event and
//! carries out the returned actions.

use super::ad::{Ad, Faction, Message, PartyId, Role, compatible};
use super::lead::{choose_party, invite_preference, plan_invites};
use super::party::{PARTY_SIZE, Party, party_id};
use super::sample::Sample;
use super::widen;
use crate::config::{
    AD_CUTOFF, AD_EXPIRY, AD_KEEPALIVE, BAN, CHECKPOINT_1, CHECKPOINT_2, CONFIRM_WINDOW, EPOCH,
    INVITE_WINDOW, READY_WINDOW, SAMPLE_SIZE,
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
    /// A leader says the party it invited me to is formed.
    AnnounceReceived {
        topic: TopicId,
        from: PeerId,
        epoch: u64,
        members: [PeerId; PARTY_SIZE],
        party_id: PartyId,
    },
    /// A leader invites me to its party.
    InviteReceived {
        topic: TopicId,
        from: PeerId,
        epoch: u64,
        ad: Ad,
    },
    /// An invitee accepted my invitation.
    AcceptReceived {
        topic: TopicId,
        from: PeerId,
        epoch: u64,
    },
    ReadyReceived {
        topic: TopicId,
        from: PeerId,
        party_id: PartyId,
        /// The sender's own ad.
        ad: Ad,
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
    /// A search ended (ADR 0027). For the host to record; the engine keeps nothing of it.
    SearchEnded(SearchFacts),
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

/// How many other players one node saw, by role. `players` is the sum of the three roles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RoleCounts {
    pub players: u32,
    pub tanks: u32,
    pub healers: u32,
    pub damage: u32,
}

impl RoleCounts {
    fn add(&mut self, role: Role) {
        self.players += 1;
        match role {
            Role::Tank => self.tanks += 1,
            Role::Healer => self.healers += 1,
            Role::Damage => self.damage += 1,
        }
    }

    /// Each count raised to at least the other's. `players` is recomputed from the roles so it
    /// stays the sum (a per-role peak need not come from one moment).
    fn peak_with(&mut self, other: &RoleCounts) {
        self.tanks = self.tanks.max(other.tanks);
        self.healers = self.healers.max(other.healers);
        self.damage = self.damage.max(other.damage);
        self.players = self.players.max(other.players);
    }
}

/// What this node observed during a search (ADR 0026). Self-reported claims only (ADR 0003): no
/// peer id and no name is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchStats {
    /// Other players in the samples now that are compatible under the stage's rules and not
    /// banned. In the frozen end snapshot this equals `peak`.
    pub current: RoleCounts,
    /// The most of each role seen at one tick over the search so far.
    pub peak: RoleCounts,
    /// A sample reached its cap, so the counts are a slice of the topic.
    pub sample_full: bool,
    pub own_role: Role,
    /// The highest widening stage the search reached.
    pub stage_reached: u8,
    /// Time since the search began (frozen at the limit in the end snapshot).
    pub elapsed: Duration,
    /// Attempt counters (ADR 0026), counts only. Frozen with the rest in the end snapshot.
    pub attempts: Attempts,
    /// The counts at the first checkpoint (`CHECKPOINT_1`) once this search has reached it while
    /// still searching; `None` before (ADR 0027, estimator). Frozen with the rest when ended.
    pub checkpoint: Option<Checkpoint>,
}

/// What this node did and received while looking for a party. Everything here is something the
/// engine really sent or received; a silent peer and a peer that preferred a better-ranked
/// leader look the same and are not told apart. No peer id is kept.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Attempts {
    /// Invitations this node sent as a leader (one per invited peer).
    pub invites_sent: u32,
    /// Distinct epochs (rounds) in which this node sent invitations. One round's silence is not
    /// a pattern: an accept takes up to 2 s to come back (ADR 0026).
    pub invite_epochs: u32,
    /// Accepts received for those invitations (once per peer and epoch).
    pub accepts_got: u32,
    /// Invitations this node received as an invitee and kept.
    pub invites_got: u32,
    /// Distinct peers whose ads left the sample because a party-taken notice named them.
    pub taken_seen: u32,
    /// Parties that fell apart because a member did not answer.
    pub drops: u32,
    /// Parties declined or left unconfirmed (not a failure).
    pub declines: u32,
}

/// How a search ended, as the engine knows it (ADR 0027). The host maps `Stopped` to `Ignored`
/// when the stop was not the player's choice (character change, clearing the character).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOutcome {
    /// Every member confirmed (`Mode::Final`).
    Formed,
    /// The time limit passed.
    Expired,
    /// The player stopped the search.
    Stopped,
    /// Ended by a restart or something that is not a result.
    Ignored,
}

/// What the role counts looked like at a checkpoint (`CHECKPOINT_1`, `CHECKPOINT_2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checkpoint {
    /// Compatible players seen at that tick (not the peak).
    pub counts: RoleCounts,
    /// The widening stage in force.
    pub stage: u8,
    pub sample_full: bool,
}

/// A plain record of one finished search. Self-reported counts only (ADR 0003): no peer id,
/// no name and no zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFacts {
    pub outcome: SearchOutcome,
    /// From the engine's own `now` values, never below zero.
    pub elapsed: Duration,
    pub stage_reached: u8,
    pub own_role: Role,
    pub level: u8,
    /// Topics chosen at the start (one per dungeon), extra widening topics not counted.
    pub topic_count: usize,
    /// Each role's own maximum over the search (not the roles at one moment).
    pub peak: RoleCounts,
    /// Offset from the search start of the first commit, if a party ever formed.
    pub first_formed_at: Option<Duration>,
    /// Parties that fell apart for a reason other than a decline or a Confirm timeout.
    pub drops: u32,
    /// Parties declined or left unconfirmed: the player's choice, not a failure.
    pub declines: u32,
    /// `None` when the search ended before the checkpoint, never zeros.
    pub checkpoint_1: Option<Checkpoint>,
    pub checkpoint_2: Option<Checkpoint>,
    /// Invitations sent as a leader, accepts got for them, and distinct peers seen as taken.
    pub invites_sent: u32,
    pub accepts_got: u32,
    pub taken_seen: u32,
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
    /// Observations of the running search, or the frozen snapshot while `Expired`. `None`
    /// otherwise (idle, final, or after the player stopped).
    pub stats: Option<SearchStats>,
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
    own_ad: Option<Ad>,
    last_epoch: Option<u64>,
    /// The party I am building as a leader (a tank). At most one per epoch.
    lead: Option<Lead>,
    /// Invitations received, by the epoch they name, waiting for the decision time.
    inbox: BTreeMap<u64, BTreeMap<PeerId, (TopicId, Ad)>>,
    /// The latest epoch I decided invitations for (I answer each epoch once).
    decided: Option<u64>,
    /// The leader I accepted: (topic, epoch, leader). Only its announcement can commit me.
    lease: Option<(TopicId, u64, PeerId)>,
    /// Most of each role seen at once so far, and whether any sample was full.
    peak: RoleCounts,
    peak_full: bool,
    peak_stage: u8,
    first_formed_at: Option<Duration>,
    drops: u32,
    declines: u32,
    invites_sent: u32,
    /// The latest epoch an invitation went out in, and how many epochs had one.
    last_invite_epoch: Option<u64>,
    invite_epochs: u32,
    accepts_got: u32,
    invites_got: u32,
    /// Peers whose ads a taken notice removed (counted once each; only the count leaves).
    taken_seen: BTreeSet<PeerId>,
    checkpoint_1: Option<Checkpoint>,
    checkpoint_2: Option<Checkpoint>,
}

/// A leader's open invitations for one epoch.
struct Lead {
    topic: TopicId,
    epoch: u64,
    /// The invited peers with their ads as the leader saw them.
    invited: BTreeMap<PeerId, Ad>,
    accepted: BTreeSet<PeerId>,
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
    /// The stats frozen when a search expired. Kept until the next `start` or a player stop.
    snapshot: Option<SearchStats>,
}

const MAX_EARLY: usize = 64;
/// Leaders an invitee keeps invitations from per epoch. More are ignored.
const MAX_INVITES_PER_EPOCH: usize = 64;

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
            snapshot: None,
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
            Event::UserStop => {
                self.end_search(now, SearchOutcome::Stopped, out);
                self.stop(out);
                // A stop by the player shows no end summary.
                self.snapshot = None;
            }
            Event::Tick => self.tick(now, out),
            Event::AdReceived { topic, ad } => {
                self.on_ad(now, topic, ad, out);
            }
            Event::TakenReceived {
                topic,
                peer,
                counter,
            } => {
                let me = self.me;
                if let Some(s) = self.search.as_mut()
                    && let Some(sample) = s.samples.get_mut(&topic)
                    && sample.mark_taken(peer, counter)
                    && peer != me
                {
                    s.taken_seen.insert(peer);
                }
            }
            Event::AnnounceReceived {
                topic,
                from,
                epoch,
                members,
                party_id: pid,
            } => self.on_announce(now, topic, from, epoch, members, pid, out),
            Event::InviteReceived {
                topic,
                from,
                epoch,
                ad,
            } => self.on_invite(now, topic, from, epoch, ad),
            Event::AcceptReceived { topic, from, epoch } => {
                self.on_accept(now, topic, from, epoch, out)
            }
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
        self.end_search(now, SearchOutcome::Ignored, out);
        self.stop(out);
        self.snapshot = None;
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
            own_ad: None,
            last_epoch: None,
            lead: None,
            inbox: BTreeMap::new(),
            decided: None,
            lease: None,
            peak: RoleCounts::default(),
            peak_full: false,
            peak_stage: 0,
            first_formed_at: None,
            drops: 0,
            declines: 0,
            invites_sent: 0,
            last_invite_epoch: None,
            invite_epochs: 0,
            accepts_got: 0,
            invites_got: 0,
            taken_seen: BTreeSet::new(),
            checkpoint_1: None,
            checkpoint_2: None,
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
        self.record_peak(now);
        self.record_checkpoints(now);
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
            // `stop` drops the search, so freeze what it observed first.
            let snapshot = self.stats(now).map(|mut st| {
                st.current = st.peak;
                st
            });
            self.end_search(now, SearchOutcome::Expired, out);
            self.stop(out);
            self.snapshot = snapshot;
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
        self.decide_invites(now, out);
    }

    /// Once per epoch a tank invites a party from its own sample (ADR 0025). Everyone else waits
    /// for invitations (`decide_invites`).
    fn run_epoch(&mut self, now: Duration, epoch: u64, out: &mut Vec<Action>) {
        let epoch_start = Duration::from_secs(epoch * EPOCH.as_secs());
        let cutoff = epoch_start.saturating_sub(AD_CUTOFF);
        let me = self.me;
        let bans = self.bans.clone();
        {
            let s = self.search.as_mut().expect("searching");
            s.last_epoch = Some(epoch);
            s.lead = None;
            s.inbox.retain(|e, _| e + 1 >= epoch);
            for sample in s.samples.values_mut() {
                sample.expire(now);
            }
        }
        let Some(own) = self.search.as_ref().and_then(|s| s.own_ad.clone()) else {
            return;
        };
        if own.role != Role::Tank {
            return;
        }
        let topics: Vec<TopicId> = self
            .search
            .as_ref()
            .unwrap()
            .joined
            .iter()
            .copied()
            .collect();
        let mut found: Option<(TopicId, BTreeMap<PeerId, Ad>)> = None;
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
            let invites = plan_invites(epoch, &own, &cands, min_level);
            if invites.is_empty() {
                continue;
            }
            let invited = cands
                .iter()
                .filter(|a| invites.contains(&a.peer))
                .map(|a| (a.peer, (*a).clone()))
                .collect();
            found = Some((topic, invited));
            break;
        }
        let Some((topic, invited)) = found else {
            return;
        };
        let s = self.search.as_mut().unwrap();
        s.invites_sent += invited.len() as u32;
        if s.last_invite_epoch != Some(epoch) {
            s.last_invite_epoch = Some(epoch);
            s.invite_epochs += 1;
        }
        for peer in invited.keys() {
            out.push(Action::SendMessage {
                topic,
                to: Some(*peer),
                msg: Message::Invite {
                    peer: me,
                    epoch,
                    ad: own.clone(),
                },
            });
        }
        self.search.as_mut().unwrap().lead = Some(Lead {
            topic,
            epoch,
            invited,
            accepted: BTreeSet::new(),
        });
    }

    // ---- agreeing on a party ----------------------------------------------------------

    /// An invitation from a leader. Kept until the decision time of its epoch.
    fn on_invite(&mut self, now: Duration, topic: TopicId, from: PeerId, epoch: u64, ad: Ad) {
        if self.mode != Mode::Searching
            || from == self.me
            || ad.peer != from
            || ad.role != Role::Tank
            || self.is_banned(&from, now)
        {
            return;
        }
        let cur = now.as_secs() / EPOCH.as_secs();
        // An invitation for the previous or next epoch is kept for one epoch.
        if epoch + 1 < cur || epoch > cur + 1 {
            return;
        }
        let min_level = self.min_level(&topic);
        let s = self.search.as_mut().unwrap();
        let Some(own) = s.own_ad.as_ref() else {
            return;
        };
        if !s.joined.contains(&topic)
            || own.role == Role::Tank
            || !compatible(own, &ad, own.stage.min(ad.stage), min_level)
            || s.decided.is_some_and(|d| epoch <= d)
        {
            return;
        }
        let group = s.inbox.entry(epoch).or_default();
        let fresh = !group.contains_key(&from);
        if group.len() < MAX_INVITES_PER_EPOCH || !fresh {
            group.insert(from, (topic, ad));
            if fresh {
                s.invites_got += 1;
            }
        }
    }

    /// Answer the invitations of every epoch whose decision time has come: accept the leader
    /// with the lowest rank, ignore the others. One answer per epoch, and the accepted leader
    /// is the only one whose announcement can commit this peer.
    fn decide_invites(&mut self, now: Duration, out: &mut Vec<Action>) {
        let me = self.me;
        let s = self.search.as_mut().expect("searching");
        let due: Vec<u64> = s
            .inbox
            .keys()
            .copied()
            .filter(|e| now >= Duration::from_secs(e * EPOCH.as_secs()) + INVITE_WINDOW)
            .collect();
        for e in due {
            let group = s.inbox.remove(&e).unwrap_or_default();
            if s.decided.is_some_and(|d| e <= d) {
                continue;
            }
            s.decided = Some(e);
            let best = group
                .into_iter()
                .min_by_key(|(leader, _)| invite_preference(e, leader));
            if let Some((leader, (topic, _))) = best {
                s.lease = Some((topic, e, leader));
                out.push(Action::SendMessage {
                    topic,
                    to: Some(leader),
                    msg: Message::Accept { peer: me, epoch: e },
                });
            }
        }
    }

    /// An invitee accepted. As soon as the acceptors make a complete compatible party, announce
    /// it to them and commit.
    fn on_accept(
        &mut self,
        now: Duration,
        topic: TopicId,
        from: PeerId,
        epoch: u64,
        out: &mut Vec<Action>,
    ) {
        if self.mode != Mode::Searching {
            return;
        }
        let me = self.me;
        let min_level = self.min_level(&topic);
        let s = self.search.as_mut().unwrap();
        let Some(own) = s.own_ad.clone() else {
            return;
        };
        let Some(lead) = s.lead.as_mut() else {
            return;
        };
        if lead.topic != topic || lead.epoch != epoch || !lead.invited.contains_key(&from) {
            return;
        }
        if lead.accepted.insert(from) {
            s.accepts_got += 1;
        }
        let lead = s.lead.as_mut().expect("checked above");
        let acc: Vec<&Ad> = lead.accepted.iter().map(|p| &lead.invited[p]).collect();
        let Some(party) = choose_party(&topic, epoch, &own, &acc, min_level) else {
            return;
        };
        for m in party.members {
            if m != me {
                out.push(Action::SendMessage {
                    topic,
                    to: Some(m),
                    msg: Message::PartyAnnounce {
                        peer: me,
                        epoch,
                        members: party.members,
                        party_id: party.id,
                    },
                });
            }
        }
        self.commit_to(now, topic, party, out);
    }

    /// The leader I accepted says the party is formed.
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
        let mut sorted = members;
        sorted.sort();
        if sorted != members || sorted.windows(2).any(|w| w[0] == w[1]) {
            return;
        }
        if !members.contains(&from)
            || !members.contains(&self.me)
            || party_id(&topic, epoch, &members) != pid
        {
            return;
        }
        let s = self.search.as_ref().unwrap();
        if s.lease != Some((topic, epoch, from)) || !s.joined.contains(&topic) {
            return; // I did not accept this leader, so I cannot be committed to its party.
        }
        let party = Party {
            id: pid,
            epoch,
            members,
        };
        self.commit_to(now, topic, party, out);
    }

    fn commit_to(&mut self, now: Duration, topic: TopicId, party: Party, out: &mut Vec<Action>) {
        let me = self.me;
        let mut names = BTreeMap::new();
        let mut details = BTreeMap::new();
        {
            let s = self.search.as_mut().unwrap();
            s.first_formed_at
                .get_or_insert(now.saturating_sub(s.search_start));
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
            // Members are taken: drop them from every sample. Stop inviting and answering.
            for sample in s.samples.values_mut() {
                for m in party.members {
                    if m != me {
                        sample.remove(&m);
                    }
                }
            }
            s.lead = None;
            s.inbox.clear();
            s.lease = None;
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
        if let Some(own) = self.search.as_ref().and_then(|s| s.own_ad.clone()) {
            names.insert(me, (own.first_name.clone(), own.surname.clone()));
            details.insert(me, details_of(&own));
            out.push(self.msg_to_all(
                topic,
                Message::Ready {
                    peer: me,
                    party_id: party.id,
                    nonce: 0,
                    ad: own,
                },
            ));
        }
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
                // A member's own ad is the source for its name and details.
                if let Event::ReadyReceived { ad, .. } = &ev
                    && ad.peer == from
                {
                    c.names
                        .insert(from, (ad.first_name.clone(), ad.surname.clone()));
                    c.details.insert(from, details_of(ad));
                }
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
        self.end_search(now, SearchOutcome::Formed, out);
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
        if let Some(s) = self.search.as_mut() {
            match cause {
                DropCause::Unreachable => s.drops += 1,
                DropCause::Declined | DropCause::Timeout => s.declines += 1,
            }
        }
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
            s.lead = None;
            s.inbox.clear();
            s.lease = None;
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

    // ---- observations -----------------------------------------------------------------

    /// The other players this node sees now: fresh, not banned, and compatible with the own ad
    /// under the stricter stage of the pair (the rule `on_ad` and party building use). A peer in
    /// several topics counts once. Also whether any sample is at its cap.
    fn observe(&self, s: &Search, now: Duration) -> (RoleCounts, bool) {
        let mut counts = RoleCounts::default();
        let mut full = false;
        let mut counted = BTreeSet::new();
        for (topic, sample) in &s.samples {
            full |= sample.is_full();
            let Some(own) = s.own_ad.as_ref() else {
                continue;
            };
            let min_level = self.min_level(topic);
            for e in sample.entries() {
                let ad = &e.ad;
                if ad.peer == self.me
                    || now.saturating_sub(e.last_seen) >= AD_EXPIRY
                    || self.is_banned(&ad.peer, now)
                    || !compatible(own, ad, own.stage.min(ad.stage), min_level)
                {
                    continue;
                }
                if counted.insert(ad.peer) {
                    counts.add(ad.role);
                }
            }
        }
        (counts, full)
    }

    fn record_peak(&mut self, now: Duration) {
        let Some(s) = self.search.as_ref() else {
            return;
        };
        let (current, full) = self.observe(s, now);
        let s = self.search.as_mut().expect("checked");
        s.peak.peak_with(&current);
        s.peak_full |= full;
        s.peak_stage = s.peak_stage.max(s.stage);
    }

    /// Take the checkpoint counts on the first searching tick at or after each offset from the
    /// search start. Not while a party is committed: the members are out of the samples then, so
    /// the counts would not describe an open search. A search that ends first keeps `None`.
    fn record_checkpoints(&mut self, now: Duration) {
        if self.mode != Mode::Searching {
            return;
        }
        let Some(s) = self.search.as_ref() else {
            return;
        };
        let elapsed = now.saturating_sub(s.search_start);
        let want1 = s.checkpoint_1.is_none() && elapsed >= CHECKPOINT_1;
        let want2 = s.checkpoint_2.is_none() && elapsed >= CHECKPOINT_2;
        if !want1 && !want2 {
            return;
        }
        let (counts, full) = self.observe(s, now);
        let cp = Checkpoint {
            counts,
            stage: s.stage,
            sample_full: full,
        };
        let s = self.search.as_mut().expect("checked");
        if want1 {
            s.checkpoint_1 = Some(cp);
        }
        if want2 {
            s.checkpoint_2 = Some(cp);
        }
    }

    /// Emit the facts of the running search, if any. Call before the search is dropped.
    fn end_search(&self, now: Duration, outcome: SearchOutcome, out: &mut Vec<Action>) {
        let (Some(s), Some(st)) = (self.search.as_ref(), self.stats(now)) else {
            return;
        };
        out.push(Action::SearchEnded(SearchFacts {
            outcome,
            elapsed: st.elapsed,
            stage_reached: st.stage_reached,
            own_role: st.own_role,
            level: s.profile.level,
            topic_count: s.topics.len(),
            peak: st.peak,
            first_formed_at: s.first_formed_at,
            drops: s.drops,
            declines: s.declines,
            checkpoint_1: s.checkpoint_1,
            checkpoint_2: s.checkpoint_2,
            invites_sent: s.invites_sent,
            accepts_got: s.accepts_got,
            taken_seen: s.taken_seen.len() as u32,
        }));
    }

    /// The stats of the running search, with the peak raised to the current counts.
    fn stats(&self, now: Duration) -> Option<SearchStats> {
        let s = self.search.as_ref()?;
        let (current, full) = self.observe(s, now);
        let mut peak = s.peak;
        peak.peak_with(&current);
        Some(SearchStats {
            current,
            peak,
            sample_full: s.peak_full || full,
            own_role: s.profile.role,
            stage_reached: s.peak_stage.max(s.stage),
            elapsed: now.saturating_sub(s.search_start),
            attempts: Attempts {
                invites_sent: s.invites_sent,
                invite_epochs: s.invite_epochs,
                accepts_got: s.accepts_got,
                invites_got: s.invites_got,
                taken_seen: s.taken_seen.len() as u32,
                drops: s.drops,
                declines: s.declines,
            },
            checkpoint: s.checkpoint_1,
        })
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
            stats: if self.mode == Mode::Expired {
                self.snapshot
            } else {
                self.stats(now)
            },
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
