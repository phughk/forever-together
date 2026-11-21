//! `LiveCore`: the running app behind the web server's `AppCore` trait. One task owns the
//! engine, the network and the character; commands and network events are handled one at a
//! time, and every change republishes the `AppState` snapshot.
//!
//! The task runs on its own single-threaded runtime in a dedicated thread, because the
//! `Network` trait's futures are not `Send`. Requirements on a `Network` implementation:
//! `next_event` must be cancel-safe (it is dropped whenever a command or a tick arrives first).

pub mod memnet;

#[cfg(test)]
mod history_tests;
#[cfg(test)]
mod tests;

use crate::code::{self, Character, CodeError};
use crate::config::{CONFIRM_WINDOW, READY_WINDOW, Settings, WIDEN_STEP};
use crate::dungeons::{self, Dungeon};
use crate::engine::state::{
    Action, Engine, Event, Mode, Profile, SearchFacts, SearchOutcome, TopicSearch, View,
};
use crate::engine::{DropCause, Message, NetError, Network, PartyId, apply_network, diagnose};
use crate::history;
use crate::identity::PeerId;
use crate::server::core::*;
use crate::store::{KEY_CHARACTER, Store, get_json, put_json};
use axum::http::StatusCode;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot, watch};

pub type Clock = Arc<dyn Fn() -> Duration + Send + Sync>;

pub fn system_clock() -> Clock {
    Arc::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
    })
}

type Reply<T> = oneshot::Sender<Result<T, CoreError>>;

enum Cmd {
    SetCharacter(String, Reply<CharacterView>),
    ClearCharacter(Reply<()>),
    ClearHistory(Reply<()>),
    Dungeons(oneshot::Sender<DungeonsResponse>),
    Start(Vec<String>, Reply<()>),
    Stop(Reply<()>),
    Confirm(Reply<()>),
    Decline(Reply<()>),
    UpdateSettings(SettingsPatch, Reply<SettingsView>),
}

pub struct CoreDeps {
    pub store: Arc<dyn Store>,
    pub me: PeerId,
    pub clock: Clock,
    /// Peer count and bootstrap source, published by the network layer.
    pub net_status: watch::Receiver<NetworkView>,
}

pub struct LiveCore {
    cmd: mpsc::UnboundedSender<Cmd>,
    state: Arc<watch::Sender<AppState>>,
}

fn ms(d: Duration) -> u64 {
    d.as_millis() as u64
}

fn role_view(r: Option<code::Role>) -> RoleView {
    match r {
        Some(code::Role::Tank) => RoleView::Tank,
        Some(code::Role::Healer) => RoleView::Healer,
        Some(code::Role::Damage) => RoleView::Damage,
        None => RoleView::Unknown,
    }
}

fn class_name(id: u8) -> Option<&'static str> {
    Some(match id {
        1 => "Warrior",
        2 => "Paladin",
        3 => "Hunter",
        4 => "Rogue",
        5 => "Priest",
        6 => "Death Knight",
        7 => "Shaman",
        8 => "Mage",
        9 => "Warlock",
        10 => "Monk",
        11 => "Druid",
        12 => "Demon Hunter",
        13 => "Evoker",
        _ => return None,
    })
}

fn race_name(id: u8) -> Option<&'static str> {
    Some(match id {
        1 => "Human",
        2 => "Orc",
        3 => "Dwarf",
        4 => "Night Elf",
        5 => "Undead",
        6 => "Tauren",
        7 => "Gnome",
        8 => "Troll",
        10 => "Blood Elf",
        11 => "Draenei",
        _ => return None,
    })
}

fn character_view(c: &Character, table: &[Dungeon]) -> CharacterView {
    let zone = c.zone.to_ascii_lowercase();
    CharacterView {
        first_name: c.first_name.clone(),
        surname: c.surname.clone(),
        faction: match c.faction {
            code::Faction::Alliance => FactionView::Alliance,
            code::Faction::Horde => FactionView::Horde,
        },
        class_name: class_name(c.class).map(str::to_owned),
        race_name: race_name(c.race).map(str::to_owned),
        level: c.level,
        item_level: c.item_level,
        role: role_view(c.role),
        realm: c.realm.clone(),
        realm_name: code::realm_name(&c.realm),
        region: c.region,
        region_name: code::region_name(c.region),
        zone: c.zone.clone(),
        zone_known: !zone.is_empty() && table.iter().any(|d| d.zone.to_ascii_lowercase() == zone),
        created_at: c.created_minutes as u64 * 60_000,
    }
}

fn code_error(e: CodeError) -> CoreError {
    CoreError::new(StatusCode::UNPROCESSABLE_ENTITY, e.name(), e.to_string())
}

fn settings_view(s: &Settings) -> SettingsView {
    SettingsView {
        lan_mode: s.lan_mode,
        lan_address: s.lan_address.clone(),
        known_peers_size: s.known_peers_cache,
        trusted_peers: s.trusted_peers.clone(),
    }
}

/// An opaque id for a party's member set, for the web app's keys. Not the wire `PartyId`.
fn party_key(members: &[PeerId]) -> String {
    let mut sorted: Vec<&PeerId> = members.iter().collect();
    sorted.sort();
    let mut h = Sha256::new();
    for m in sorted {
        h.update(m.0);
    }
    data_encoding::HEXLOWER.encode(&h.finalize()[..16])
}

/// What the model remembers about the party between publishes.
#[derive(Default)]
struct PartyTrack {
    /// Id and time (ms) the confirming party was first seen, for the Confirm deadline.
    seen: Option<(String, u64)>,
    /// The last confirming party, to turn its disappearance into a `dropped` view.
    last_confirming: Option<PartyView>,
    dropped: Option<(PartyView, u64)>,
}

/// How long a dropped party stays on screen.
const DROPPED_SHOWN_MS: u64 = 20_000;

/// Map the engine's view of a party. Pure so it can be tested directly. Details the engine's
/// `View` does not carry (other members' role, level and item level, who declined) are
/// reported as unknown.
fn party_view(
    view: &View,
    me: &PeerId,
    my: Option<&Character>,
    seen_at: u64,
    track: &mut PartyTrack,
) -> Option<PartyView> {
    if !matches!(view.mode, Mode::Committed | Mode::Final) || view.party.is_empty() {
        return None;
    }
    let peers: Vec<PeerId> = view.party.iter().map(|m| m.peer).collect();
    let id = party_key(&peers);
    let final_ = view.mode == Mode::Final;
    let deadline = if final_ || !view.confirm_prompt {
        None
    } else {
        let first = match &track.seen {
            Some((sid, t)) if *sid == id => *t,
            _ => {
                track.seen = Some((id.clone(), seen_at));
                seen_at
            }
        };
        Some(first + CONFIRM_WINDOW.as_millis() as u64)
    };
    let members = view
        .party
        .iter()
        .map(|m| {
            let is_self = m.peer == *me;
            PartyMember {
                peer_id: m.peer.to_hex(),
                first_name: if final_ {
                    m.name.as_ref().map(|n| n.0.clone())
                } else {
                    None
                },
                surname: if final_ {
                    m.name.as_ref().map(|n| n.1.clone())
                } else {
                    None
                },
                role: match (is_self, my, m.details) {
                    (true, Some(c), _) => role_view(c.role),
                    (_, _, Some(d)) => role_view(Some(d.role)),
                    _ => RoleView::Unknown,
                },
                level: match (is_self, my, m.details) {
                    (true, Some(c), _) => c.level,
                    (_, _, Some(d)) => d.level,
                    _ => 0,
                },
                item_level: match (is_self, my, m.details) {
                    (true, Some(c), _) => c.item_level,
                    (_, _, Some(d)) => d.item_level,
                    _ => 0,
                },
                is_self,
                ready: m.ready,
                confirmed: m.confirmed,
                declined: false,
            }
        })
        .collect();
    Some(PartyView {
        id,
        phase: if final_ {
            PartyPhase::Final
        } else {
            PartyPhase::Confirming
        },
        drop_reason: None,
        members,
        confirm_deadline: deadline,
    })
}

struct Model {
    store: Arc<dyn Store>,
    me: PeerId,
    clock: Clock,
    engine: Engine,
    table: Vec<Dungeon>,
    character: Option<Character>,
    settings: Settings,
    slugs: Vec<String>,
    started_at: Option<u64>,
    last_stage_clock: Option<u64>,
    track: PartyTrack,
    hb: Heartbeats,
    net_status: watch::Receiver<NetworkView>,
    published: AppState,
    /// Set while the host ends a search for its own reasons (character change), so the engine's
    /// `Stopped` is not a result and is not recorded.
    stop_is_not_a_result: bool,
    /// The readable history entries, loaded once and refreshed after each recorded entry or
    /// cleared, so the 1 s tick never reads the store. Empty when a read failed: no estimate.
    history_cache: Vec<history::Entry>,
}

impl Model {
    /// Reload the cached entries from the store. A failed read means no estimate.
    fn reload_history(&mut self) {
        self.history_cache = match history::load(self.store.as_ref()) {
            Ok(rows) => rows.into_iter().map(|(_, e)| e).collect(),
            Err(e) => {
                tracing::warn!("search history not read: {e}");
                Vec::new()
            }
        };
    }

    /// The banded outlook for the running search, from the cached entries. Only once the search
    /// has reached its 55 s checkpoint (see `history::estimate`).
    fn outlook_band(&self, view: &View) -> Option<history::estimate::Band> {
        let cp = view.stats?.checkpoint?;
        let dungeon = self.slugs.first()?;
        history::estimate::estimate(
            &self.history_cache,
            &history::estimate::Observation {
                counts: cp.counts,
                own_role: view.stats?.own_role,
                dungeon,
            },
        )
        .band()
    }

    fn now(&self) -> Duration {
        (self.clock)()
    }

    fn app_state(&mut self) -> AppState {
        let now = self.now();
        let now_ms = ms(now);
        let view = self.engine.view(now);
        let searching_mode = matches!(view.mode, Mode::Searching | Mode::Committed);
        let status = match view.mode {
            Mode::Searching | Mode::Committed => SearchStatus::Searching,
            Mode::Expired => SearchStatus::LimitReached,
            Mode::Idle | Mode::Final => SearchStatus::Idle,
        };
        let active = !matches!(view.mode, Mode::Idle);
        let stage_clock = if searching_mode {
            match view.next_stage_in {
                Some(d) => {
                    let elapsed = WIDEN_STEP * (view.stage as u32 + 1) - d;
                    let c = now_ms.saturating_sub(ms(elapsed));
                    self.last_stage_clock = Some(c);
                    Some(c)
                }
                None => self.last_stage_clock.or_else(|| {
                    self.started_at
                        .map(|s| s + ms(WIDEN_STEP * view.stage as u32))
                }),
            }
        } else {
            None
        };
        let extra = if view.stage >= 4 {
            self.extra_topics().len()
        } else {
            0
        };

        // Party and its drop.
        let mut party = party_view(
            &view,
            &self.me,
            self.character.as_ref(),
            now_ms,
            &mut self.track,
        );
        match &party {
            Some(p) if p.phase == PartyPhase::Confirming => {
                self.track.last_confirming = Some(p.clone());
                self.track.dropped = None;
            }
            Some(_) => {
                self.track.last_confirming = None;
                self.track.dropped = None;
            }
            None => {
                if let Some(mut last) = self.track.last_confirming.take()
                    && view.mode == Mode::Searching
                {
                    last.phase = PartyPhase::Dropped;
                    last.confirm_deadline = None;
                    let info = view.last_drop;
                    last.drop_reason = Some(match info.map(|i| i.cause) {
                        Some(DropCause::Declined) => DropReason::Declined,
                        Some(DropCause::Unreachable) => DropReason::Unreachable,
                        _ => DropReason::Timeout,
                    });
                    if let Some(i) = info
                        && i.cause == DropCause::Declined
                        && let Some(p) = i.peer
                    {
                        for member in last.members.iter_mut() {
                            member.declined = member.peer_id == p.to_hex();
                        }
                    }
                    self.track.dropped = Some((last, now_ms + DROPPED_SHOWN_MS));
                }
                self.track.seen = None;
                if view.mode != Mode::Searching {
                    self.track.dropped = None;
                }
                if let Some((d, until)) = &self.track.dropped {
                    if now_ms < *until {
                        party = Some(d.clone());
                    } else {
                        self.track.dropped = None;
                    }
                }
            }
        }

        let network = self.net_status.borrow().clone();
        let band = self.outlook_band(&view);
        let diagnosis = diagnosis_view(&view, network.peers, band);

        AppState {
            server_now: now_ms,
            character: self
                .character
                .as_ref()
                .map(|c| character_view(c, &self.table)),
            search: SearchView {
                status,
                dungeons: if active { self.slugs.clone() } else { vec![] },
                started_at: if active { self.started_at } else { None },
                stage: view.stage,
                stage_clock_started_at: stage_clock,
                searching: (view.mode == Mode::Searching).then_some(view.peers_seen as u32),
                topics: if active {
                    (self.slugs.len() + extra) as u32
                } else {
                    0
                },
                diagnosis,
            },
            party,
            network,
            settings: settings_view(&self.settings),
        }
    }

    fn extra_topics(&self) -> Vec<TopicSearch> {
        let Some(c) = &self.character else {
            return vec![];
        };
        dungeons::recommend(c, &self.table)
            .into_iter()
            .filter(|d| !self.slugs.contains(&d.slug))
            .map(|d| topic_search(c, d))
            .collect()
    }

    fn recommended_slugs(&self) -> Vec<String> {
        self.character
            .as_ref()
            .map(|c| {
                dungeons::recommend(c, &self.table)
                    .into_iter()
                    .map(|d| d.slug.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The search diagnosis from the engine's own observations (ADR 0026). Only while a search
/// runs (also while a party is committed, which the status calls searching) or after it
/// expired.
fn diagnosis_view(
    view: &View,
    network_peers: u32,
    band: Option<history::estimate::Band>,
) -> Option<DiagnosisView> {
    let ended = match view.mode {
        Mode::Searching | Mode::Committed => false,
        Mode::Expired => true,
        _ => return None,
    };
    let st = view.stats?;
    let c = if ended { st.peak } else { st.current };
    let seen = SeenView {
        players: c.players,
        tanks: c.tanks,
        healers: c.healers,
        damage: c.damage,
    };
    let situation = diagnose::diagnose(&diagnose::Input {
        ended,
        elapsed: st.elapsed,
        network_peers,
        seen: c,
        own_role: Some(st.own_role),
        attempts: st.attempts,
    });
    let a = st.attempts;
    // History may add one banded clause but never changes the situation (ADR 0026).
    let outlook = history::estimate::outlook(situation, ended, band).map(|b| b.code().to_string());
    Some(DiagnosisView {
        situation: situation.code().to_string(),
        outlook,
        ended,
        seen,
        own_role: Some(
            match st.own_role {
                code::Role::Tank => "tank",
                code::Role::Healer => "healer",
                code::Role::Damage => "damage",
            }
            .to_string(),
        ),
        sample_full: st.sample_full,
        network_peers,
        attempts: AttemptsView {
            invites_sent: a.invites_sent,
            accepts_got: a.accepts_got,
            invites_got: a.invites_got,
            taken_seen: a.taken_seen,
            drops: a.drops,
            declines: a.declines,
        },
    })
}

fn topic_search(c: &Character, d: &Dungeon) -> TopicSearch {
    TopicSearch {
        topic: dungeons::topics_for(c, &[d])[0],
        min_level: d.min_level.unwrap_or(0),
    }
}

/// Hand an event to the engine and carry out what it asks.
async fn feed<N: Network>(m: &mut Model, net: &mut Option<N>, ev: Event) {
    let now = m.now();
    if let Event::HeartbeatAckReceived { from, party_id } = &ev
        && m.hb.party == Some(*party_id)
    {
        m.hb.outstanding.remove(from);
    }
    let actions = m.engine.handle(now, ev);
    for a in &actions {
        if let Action::SearchEnded(facts) = a {
            record_search(m, now, facts);
        }
    }
    let rest = match net.as_mut() {
        Some(n) => match apply_network(n, actions).await {
            Ok(rest) => rest,
            Err(e) => {
                tracing::warn!("network action failed: {e}");
                Vec::new()
            }
        },
        None => actions,
    };
    // `BanLocal` is already applied inside the engine; nothing else is left for the host.
    let _ = rest;
}

/// The local offset from UTC in seconds. The standard library cannot tell, and a date-time
/// dependency is not worth it for a field nothing reads yet, so this is 0. TODO: the browser
/// knows the offset and could send it with a command (ADR 0027).
fn local_utc_offset_secs() -> i32 {
    0
}

/// Write the history entry for a finished search (ADR 0027). Never fails or stops a search: a
/// store error is logged by `history::record_facts`.
fn record_search(m: &mut Model, now: Duration, facts: &SearchFacts) {
    let mut facts = facts.clone();
    if m.stop_is_not_a_result && facts.outcome == SearchOutcome::Stopped {
        facts.outcome = SearchOutcome::Ignored;
    }
    let (Some(c), Some(dungeon)) = (m.character.as_ref(), m.slugs.first()) else {
        return;
    };
    let ctx = history::Context {
        end_utc: now.as_secs(),
        utc_offset_secs: local_utc_offset_secs(),
        region: c.region,
        realm: c.realm.clone(),
        faction: c.faction,
        dungeon: dungeon.clone(),
    };
    if history::record_facts(m.store.as_ref(), &facts, &ctx) {
        m.reload_history();
    }
}

/// How often a heartbeat goes to a member that has no outstanding one.
const HEARTBEAT_EVERY: Duration = Duration::from_secs(1);

/// Heartbeats the host sends while a party is committed (plan: 4 s reply window). A member that
/// does not answer within `READY_WINDOW` is reported to the engine as `HeartbeatTimeout`, which
/// drops the party and bans that peer locally for `BAN`. The ban is never gossiped.
#[derive(Default)]
struct Heartbeats {
    party: Option<PartyId>,
    outstanding: HashMap<PeerId, Duration>,
    last_sent: Duration,
    nonce: u64,
}

async fn heartbeat_tick<N: Network>(m: &mut Model, net: &mut Option<N>) {
    let now = m.now();
    let view = m.engine.view(now);
    let (topic, pid) = match (view.mode, view.committed) {
        (Mode::Committed, Some(c)) => c,
        _ => {
            m.hb = Heartbeats::default();
            return;
        }
    };
    if m.hb.party != Some(pid) {
        m.hb = Heartbeats {
            party: Some(pid),
            ..Heartbeats::default()
        };
    }
    let silent =
        m.hb.outstanding
            .iter()
            .find(|(_, sent)| now.saturating_sub(**sent) >= READY_WINDOW)
            .map(|(p, _)| *p);
    if let Some(peer) = silent {
        m.hb = Heartbeats::default();
        feed(m, net, Event::HeartbeatTimeout { peer }).await;
        return;
    }
    if now.saturating_sub(m.hb.last_sent) < HEARTBEAT_EVERY {
        return;
    }
    m.hb.last_sent = now;
    let Some(n) = net.as_mut() else { return };
    for member in view.party.iter().filter(|x| !x.confirmed && x.peer != m.me) {
        if m.hb.outstanding.contains_key(&member.peer) {
            continue;
        }
        m.hb.nonce += 1;
        let msg = Message::Heartbeat {
            peer: m.me,
            party_id: pid,
            nonce: m.hb.nonce,
        };
        // A failed send counts as no answer, so the peer times out like a silent one.
        let _ = n.send_direct(member.peer, topic, msg).await;
        m.hb.outstanding.insert(member.peer, now);
    }
}

fn publish(m: &mut Model, tx: &watch::Sender<AppState>) {
    let mut s = m.app_state();
    // The clock alone does not count as a change.
    let mut old = m.published.clone();
    old.server_now = s.server_now;
    if old == s {
        return;
    }
    s.server_now = ms(m.now());
    m.published = s.clone();
    tx.send_replace(s);
}

fn validate_settings(patch: SettingsPatch, current: &Settings) -> Result<Settings, CoreError> {
    let mut s = current.clone();
    if let Some(l) = patch.lan_mode {
        s.lan_mode = l;
    }
    if let Some(a) = patch.lan_address {
        let a = a.trim();
        s.lan_address = if a.is_empty() {
            None
        } else {
            let ip: std::net::Ipv4Addr = a.parse().map_err(|_| {
                CoreError::bad_request("bad_lan_address", "lan_address must be an IPv4 address")
            })?;
            Some(ip.to_string())
        };
    }
    if let Some(k) = patch.known_peers_size {
        if !(1..=100_000).contains(&k) {
            return Err(CoreError::bad_request(
                "bad_known_peers_size",
                "known_peers_size must be between 1 and 100000",
            ));
        }
        s.known_peers_cache = k;
    }
    if let Some(list) = patch.trusted_peers {
        if list.len() > 64 {
            return Err(CoreError::bad_request(
                "too_many_trusted_peers",
                "At most 64 trusted peers",
            ));
        }
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for item in list {
            let id = PeerId::from_hex(item.trim()).ok_or_else(|| {
                CoreError::bad_request(
                    "bad_peer_key",
                    "A trusted peer key is 64 hexadecimal characters",
                )
            })?;
            if seen.insert(id) {
                out.push(id.to_hex());
            }
        }
        s.trusted_peers = out;
    }
    Ok(s)
}

/// Publish the new state, then answer, so a caller that reads the state after its command
/// returns sees the effect.
fn finish<T>(m: &mut Model, state: &watch::Sender<AppState>, reply: oneshot::Sender<T>, value: T) {
    publish(m, state);
    let _ = reply.send(value);
}

async fn handle_cmd<N: Network>(
    m: &mut Model,
    net: &mut Option<N>,
    state: &watch::Sender<AppState>,
    cmd: Cmd,
) {
    match cmd {
        Cmd::SetCharacter(text, reply) => {
            let r = async {
                let c = code::decode(&text).map_err(code_error)?;
                if m.engine.mode() != Mode::Idle {
                    stop_search(m, net, false).await;
                }
                put_json(m.store.as_ref(), KEY_CHARACTER, &c)
                    .map_err(|e| CoreError::internal(e.to_string()))?;
                let v = character_view(&c, &m.table);
                m.character = Some(c);
                Ok(v)
            }
            .await;
            finish(m, state, reply, r);
        }
        Cmd::ClearCharacter(reply) => {
            stop_search(m, net, false).await;
            m.character = None;
            // A cleared character's history must not survive (ADR 0027).
            let h = history::clear(m.store.as_ref());
            m.history_cache.clear();
            let r = m
                .store
                .delete(KEY_CHARACTER)
                .map(|_| ())
                .and(h)
                .map_err(|e| CoreError::internal(e.to_string()));
            finish(m, state, reply, r);
        }
        Cmd::ClearHistory(reply) => {
            let r =
                history::clear(m.store.as_ref()).map_err(|e| CoreError::internal(e.to_string()));
            m.history_cache.clear();
            finish(m, state, reply, r);
        }
        Cmd::Dungeons(reply) => {
            let dungeons = m
                .table
                .iter()
                .map(|d| DungeonEntry {
                    slug: d.slug.clone(),
                    name: d.name.clone(),
                    level_min: d.level_min,
                    level_max: d.level_max,
                    min_level: d.min_level,
                    faction_hint: "both".into(),
                    zone: d.zone.clone(),
                })
                .collect();
            let value = DungeonsResponse {
                dungeons,
                recommended: m.recommended_slugs(),
            };
            finish(m, state, reply, value);
        }
        Cmd::Start(slugs, reply) => {
            let r = start_search(m, net, slugs).await;
            finish(m, state, reply, r);
        }
        Cmd::Stop(reply) => {
            stop_search(m, net, true).await;
            finish(m, state, reply, Ok(()));
        }
        Cmd::Confirm(reply) => {
            let r = if m.engine.mode() == Mode::Committed {
                feed(m, net, Event::UserConfirm).await;
                Ok(())
            } else {
                Err(CoreError::conflict(
                    "no_party",
                    "There is no party to confirm",
                ))
            };
            finish(m, state, reply, r);
        }
        Cmd::Decline(reply) => {
            let r = if m.engine.mode() == Mode::Committed {
                feed(m, net, Event::UserDecline).await;
                Ok(())
            } else {
                Err(CoreError::conflict(
                    "no_party",
                    "There is no party to decline",
                ))
            };
            finish(m, state, reply, r);
        }
        Cmd::UpdateSettings(patch, reply) => {
            let r = validate_settings(patch, &m.settings).and_then(|s| {
                s.save(m.store.as_ref())
                    .map_err(|e| CoreError::internal(e.to_string()))?;
                m.settings = s;
                Ok(settings_view(&m.settings))
            });
            finish(m, state, reply, r);
        }
    }
}

/// End the search. `by_player` is false when the host does it for its own reasons.
async fn stop_search<N: Network>(m: &mut Model, net: &mut Option<N>, by_player: bool) {
    if m.engine.mode() != Mode::Idle {
        m.stop_is_not_a_result = !by_player;
        feed(m, net, Event::UserStop).await;
        m.stop_is_not_a_result = false;
    }
    m.slugs.clear();
    m.started_at = None;
    m.last_stage_clock = None;
    m.track = PartyTrack::default();
}

async fn start_search<N: Network>(
    m: &mut Model,
    net: &mut Option<N>,
    slugs: Vec<String>,
) -> Result<(), CoreError> {
    let Some(c) = m.character.clone() else {
        return Err(CoreError::conflict(
            "no_character",
            "Paste a character code first",
        ));
    };
    let Some(role) = c.role else {
        return Err(CoreError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "role_unknown",
            "The addon could not tell this character's role. Pick a specialization and refresh the code.",
        ));
    };
    if net.is_none() {
        return Err(CoreError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "network_unavailable",
            "The network could not be started",
        ));
    }
    if matches!(m.engine.mode(), Mode::Committed | Mode::Final) {
        return Err(CoreError::conflict(
            "party_active",
            "A party is already forming",
        ));
    }
    let mut chosen: Vec<String> = Vec::new();
    for s in slugs {
        if !chosen.contains(&s) {
            chosen.push(s);
        }
    }
    if chosen.is_empty() {
        return Err(CoreError::bad_request(
            "no_dungeons",
            "Pick at least one dungeon",
        ));
    }
    let mut topics = Vec::new();
    for s in &chosen {
        match dungeons::find(&m.table, s) {
            Some(d) => topics.push(topic_search(&c, d)),
            None => {
                return Err(CoreError::bad_request(
                    "unknown_dungeon",
                    format!("{s} is not a known dungeon"),
                ));
            }
        }
    }
    let profile = Profile {
        role,
        level: c.level,
        item_level: c.item_level,
        faction: c.faction,
        first_name: c.first_name.clone(),
        surname: c.surname.clone(),
        zone: c.zone.clone(),
    };
    m.slugs = chosen;
    let extra = m.extra_topics();
    m.track = PartyTrack::default();
    m.started_at = Some(ms(m.now()));
    m.last_stage_clock = None;
    feed(
        m,
        net,
        Event::UserStart {
            profile,
            topics,
            extra,
        },
    )
    .await;
    Ok(())
}

async fn run<N: Network>(
    mut m: Model,
    mut net: Option<N>,
    mut rx: mpsc::UnboundedReceiver<Cmd>,
    state: Arc<watch::Sender<AppState>>,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut net_status = m.net_status.clone();
    let mut status_alive = true;
    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(c) => handle_cmd(&mut m, &mut net, &state, c).await,
                None => break,
            },
            ev = async {
                match net.as_mut() {
                    Some(n) => n.next_event().await,
                    None => std::future::pending().await,
                }
            } => match ev {
                Ok(e) => feed(&mut m, &mut net, e).await,
                Err(NetError::Closed) => {
                    tracing::error!("the network closed");
                    net = None;
                }
                Err(e) => {
                    tracing::warn!("network event error: {e}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
            _ = ticker.tick() => {
                feed(&mut m, &mut net, Event::Tick).await;
                heartbeat_tick(&mut m, &mut net).await;
            }
            r = net_status.changed(), if status_alive => {
                if r.is_err() {
                    // The status source is gone; keep the last value and stop watching it.
                    status_alive = false;
                }
            }
        }
        publish(&mut m, &state);
    }
}

impl LiveCore {
    /// Start the core. `make_net` builds the network inside the core's own runtime (iroh needs
    /// to bind sockets there). If it fails the app still runs, but cannot search.
    pub fn spawn<N, F, Fut>(deps: CoreDeps, make_net: F) -> LiveCore
    where
        N: Network + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<N, NetError>> + 'static,
    {
        let character: Option<Character> = get_json(deps.store.as_ref(), KEY_CHARACTER)
            .unwrap_or_else(|e| {
                tracing::warn!("stored character ignored: {e}");
                None
            });
        let settings = Settings::load(deps.store.as_ref()).unwrap_or_else(|e| {
            tracing::warn!("stored settings ignored: {e}");
            Settings::default()
        });
        let mut model = Model {
            store: deps.store,
            me: deps.me,
            clock: deps.clock,
            engine: Engine::new(deps.me),
            table: dungeons::load_table(),
            character,
            settings,
            slugs: vec![],
            started_at: None,
            last_stage_clock: None,
            track: PartyTrack::default(),
            hb: Heartbeats::default(),
            net_status: deps.net_status,
            published: placeholder_state(),
            stop_is_not_a_result: false,
            history_cache: Vec::new(),
        };
        model.reload_history();
        let first = model.app_state();
        model.published = first.clone();
        let state = Arc::new(watch::channel(first).0);
        let (cmd, rx) = mpsc::unbounded_channel();
        let task_state = state.clone();
        std::thread::Builder::new()
            .name("wowft-core".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("core runtime");
                rt.block_on(async move {
                    let net = match make_net().await {
                        Ok(n) => Some(n),
                        Err(e) => {
                            tracing::error!("network failed to start: {e}");
                            None
                        }
                    };
                    run(model, net, rx, task_state).await;
                });
            })
            .expect("core thread");
        LiveCore { cmd, state }
    }

    async fn ask<T>(&self, make: impl FnOnce(oneshot::Sender<T>) -> Cmd) -> Option<T> {
        let (tx, rx) = oneshot::channel();
        self.cmd.send(make(tx)).ok()?;
        rx.await.ok()
    }

    async fn ask_result<T>(&self, make: impl FnOnce(Reply<T>) -> Cmd) -> Result<T, CoreError> {
        self.ask(make)
            .await
            .unwrap_or_else(|| Err(CoreError::internal("the app core has stopped")))
    }
}

fn placeholder_state() -> AppState {
    AppState {
        server_now: 0,
        character: None,
        search: SearchView {
            status: SearchStatus::Idle,
            dungeons: vec![],
            started_at: None,
            stage: 0,
            stage_clock_started_at: None,
            searching: None,
            topics: 0,
            diagnosis: None,
        },
        party: None,
        network: NetworkView {
            peers: 0,
            bootstrap: BootstrapSource::None,
        },
        settings: SettingsView {
            lan_mode: false,
            lan_address: None,
            known_peers_size: 0,
            trusted_peers: vec![],
        },
    }
}

impl AppCore for LiveCore {
    fn state(&self) -> AppState {
        self.state.borrow().clone()
    }
    fn subscribe(&self) -> watch::Receiver<AppState> {
        self.state.subscribe()
    }
    async fn set_character(&self, code: String) -> Result<CharacterView, CoreError> {
        self.ask_result(|r| Cmd::SetCharacter(code, r)).await
    }
    async fn clear_character(&self) -> Result<(), CoreError> {
        self.ask_result(Cmd::ClearCharacter).await
    }
    async fn clear_history(&self) -> Result<(), CoreError> {
        self.ask_result(Cmd::ClearHistory).await
    }
    async fn dungeons(&self) -> DungeonsResponse {
        self.ask(Cmd::Dungeons).await.unwrap_or(DungeonsResponse {
            dungeons: vec![],
            recommended: vec![],
        })
    }
    async fn start_search(&self, dungeons: Vec<String>) -> Result<(), CoreError> {
        self.ask_result(|r| Cmd::Start(dungeons, r)).await
    }
    async fn stop_search(&self) -> Result<(), CoreError> {
        self.ask_result(Cmd::Stop).await
    }
    async fn confirm_party(&self) -> Result<(), CoreError> {
        self.ask_result(Cmd::Confirm).await
    }
    async fn decline_party(&self) -> Result<(), CoreError> {
        self.ask_result(Cmd::Decline).await
    }
    fn settings(&self) -> SettingsView {
        self.state.borrow().settings.clone()
    }
    async fn update_settings(&self, patch: SettingsPatch) -> Result<SettingsView, CoreError> {
        self.ask_result(|r| Cmd::UpdateSettings(patch, r)).await
    }
}
