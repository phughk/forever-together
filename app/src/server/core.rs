//! What the web server needs from the running app, and the wire types of the local API.
//!
//! The shapes match `src/api/types.ts`: JSON is camelCase except the settings keys, which keep
//! the plan's snake_case names. `crate::app::LiveCore` implements `AppCore` over the engine,
//! the store and a `Network`; tests use fakes.

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleView {
    Tank,
    Healer,
    Damage,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FactionView {
    Alliance,
    Horde,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterView {
    pub first_name: String,
    pub surname: String,
    pub faction: FactionView,
    pub class_name: Option<String>,
    pub race_name: Option<String>,
    pub level: u8,
    pub item_level: u16,
    pub role: RoleView,
    pub realm: String,
    /// Display name of `realm` (PvP, PvE, RP-PvP, or `realm <id>` when unknown).
    pub realm_name: String,
    pub region: u8,
    /// Display name of `region` (EU, US, ... or `region <n>` when unknown).
    pub region_name: String,
    pub zone: String,
    pub zone_known: bool,
    /// Server-clock milliseconds when the addon built the code.
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonEntry {
    pub slug: String,
    pub name: String,
    pub level_min: u8,
    pub level_max: u8,
    /// `None`: no known minimum level.
    pub min_level: Option<u8>,
    /// The table has no faction column, so this is always `both`.
    pub faction_hint: String,
    pub zone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonsResponse {
    pub dungeons: Vec<DungeonEntry>,
    pub recommended: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchStatus {
    Idle,
    Searching,
    LimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchView {
    pub status: SearchStatus,
    pub dungeons: Vec<String>,
    pub started_at: Option<u64>,
    pub stage: u8,
    pub stage_clock_started_at: Option<u64>,
    pub searching: Option<u32>,
    pub topics: u32,
    /// What this node observed while searching, or at expiry. Absent when idle (ADR 0026).
    #[serde(default)]
    pub diagnosis: Option<DiagnosisView>,
}

/// Other players this node saw that are compatible under the current stage's rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeenView {
    pub players: u32,
    pub tanks: u32,
    pub healers: u32,
    pub damage: u32,
}

/// A situation code plus the counts behind it. The web app writes the sentences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisView {
    /// One of `warming_up`, `not_connected`, `others_grouping`, `alone`, `no_tank`,
    /// `no_healer`, `short_on_damage`, `nobody_answered`, `healthy`. A String so a newer server can add codes.
    pub situation: String,
    /// The search expired: `seen` is the frozen peak, not live.
    pub ended: bool,
    pub seen: SeenView,
    /// `tank`, `healer` or `damage`; null if unknown.
    pub own_role: Option<String>,
    /// The sample hit its cap, so the counts are a slice.
    pub sample_full: bool,
    pub network_peers: u32,
    /// This node's own attempt counters (ADR 0026); frozen when `ended`. Counts only.
    #[serde(default)]
    pub attempts: AttemptsView,
}

/// What this node sent and received while looking for a party.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptsView {
    /// Invitations sent as a leader, one per invited peer.
    pub invites_sent: u32,
    /// Accepts received for them.
    pub accepts_got: u32,
    /// Invitations received as an invitee.
    pub invites_got: u32,
    /// Distinct peers whose ads left the sample because a party-taken notice named them.
    pub taken_seen: u32,
    /// Parties that fell apart because a member did not answer.
    pub drops: u32,
    /// Parties declined or left unconfirmed.
    pub declines: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartyPhase {
    Confirming,
    Final,
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropReason {
    Declined,
    Timeout,
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartyMember {
    pub peer_id: String,
    pub first_name: Option<String>,
    pub surname: Option<String>,
    pub role: RoleView,
    pub level: u8,
    pub item_level: u16,
    #[serde(rename = "self")]
    pub is_self: bool,
    pub ready: bool,
    pub confirmed: bool,
    pub declined: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartyView {
    pub id: String,
    pub phase: PartyPhase,
    pub drop_reason: Option<DropReason>,
    pub members: Vec<PartyMember>,
    pub confirm_deadline: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapSource {
    Remembered,
    PeerExchange,
    Trusted,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkView {
    pub peers: u32,
    pub bootstrap: BootstrapSource,
}

/// User settings as the web app sees them (snake_case keys).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsView {
    pub lan_mode: bool,
    /// The chosen pairing address, or null for automatic.
    pub lan_address: Option<String>,
    pub known_peers_size: usize,
    pub trusted_peers: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsPatch {
    pub lan_mode: Option<bool>,
    /// An IPv4 address, or an empty string to go back to automatic.
    pub lan_address: Option<String>,
    pub known_peers_size: Option<usize>,
    pub trusted_peers: Option<Vec<String>>,
}

impl SettingsPatch {
    /// Writes to these need a loopback caller.
    pub fn touches_loopback_only(&self) -> bool {
        self.lan_mode.is_some() || self.lan_address.is_some() || self.trusted_peers.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub server_now: u64,
    pub character: Option<CharacterView>,
    pub search: SearchView,
    pub party: Option<PartyView>,
    pub network: NetworkView,
    pub settings: SettingsView,
}

/// An error the app reports to a caller: HTTP status, a stable code and a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl CoreError {
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
        }
    }
    pub fn bad_request(code: &str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }
    pub fn conflict(code: &str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", message)
    }
}

/// The running app as the server sees it. Reads are synchronous, commands are async because
/// the implementation hands them to the task that owns the engine and the network.
pub trait AppCore: Send + Sync + 'static {
    /// The latest snapshot (`server_now` is refreshed by the server on output).
    fn state(&self) -> AppState;
    /// A receiver that changes whenever the snapshot changes.
    fn subscribe(&self) -> watch::Receiver<AppState>;
    fn set_character(
        &self,
        code: String,
    ) -> impl Future<Output = Result<CharacterView, CoreError>> + Send;
    fn clear_character(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
    /// Delete every stored search-history entry (ADR 0027). Loopback only at the route.
    fn clear_history(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn dungeons(&self) -> impl Future<Output = DungeonsResponse> + Send;
    fn start_search(
        &self,
        dungeons: Vec<String>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn stop_search(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn confirm_party(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn decline_party(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn settings(&self) -> SettingsView;
    fn update_settings(
        &self,
        patch: SettingsPatch,
    ) -> impl Future<Output = Result<SettingsView, CoreError>> + Send;
}
