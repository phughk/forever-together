//! Ads, wire messages (spec section 3) and the pair compatibility check.

use super::widen;
use crate::dungeons::TopicId;
use crate::identity::PeerId;
use serde::{Deserialize, Serialize};

pub use crate::code::{Faction, Role};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PartyId(pub [u8; 32]);

impl std::fmt::Debug for PartyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PartyId({}..)",
            data_encoding::HEXLOWER.encode(&self.0[..4])
        )
    }
}

/// A search ad. Every field is a claim by its author.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ad {
    pub peer: PeerId,
    pub counter: u16,
    pub role: Role,
    pub secondary_role: Option<Role>,
    pub level: u8,
    /// 0 when unknown.
    pub item_level: u16,
    pub faction: Faction,
    pub stage: u8,
    pub first_name: String,
    pub surname: String,
    pub zone: String,
    /// Sender's clock, informational only.
    pub created_at: u32,
}

/// True when `new` is a later counter than `old`, with 16-bit wraparound.
pub fn counter_newer(new: u16, old: u16) -> bool {
    (new.wrapping_sub(old) as i16) > 0
}

/// The messages members and searchers exchange. `net` verifies the author's signature before
/// a message reaches the engine, so the `peer` in each message is a verified author.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    Ad(Ad),
    Taken {
        peer: PeerId,
        counter: u16,
    },
    /// The leader tells the four it invited and accepted that the party is formed (direct).
    PartyAnnounce {
        peer: PeerId,
        epoch: u64,
        members: [PeerId; 5],
        party_id: PartyId,
    },
    /// The sender's app is alive and shows the prompt. Carries the sender's own ad so the other
    /// members learn its name and details from the member itself.
    Ready {
        peer: PeerId,
        party_id: PartyId,
        nonce: u64,
        ad: Ad,
    },
    Confirm {
        peer: PeerId,
        party_id: PartyId,
    },
    Decline {
        peer: PeerId,
        party_id: PartyId,
    },
    Heartbeat {
        peer: PeerId,
        party_id: PartyId,
        nonce: u64,
    },
    HeartbeatAck {
        peer: PeerId,
        party_id: PartyId,
        nonce: u64,
    },
    /// A leader (a tank) invites a peer to the party it is building (direct). Carries the
    /// leader's ad so the invitee can check compatibility without having it in its sample.
    Invite {
        peer: PeerId,
        epoch: u64,
        ad: Ad,
    },
    /// An invitee takes the best invitation it got this epoch (direct to that leader).
    Accept {
        peer: PeerId,
        epoch: u64,
    },
}

/// Pair compatibility at `stage` (the caller passes the stricter, that is the lower, of the two
/// peers' stages). Symmetric.
///
/// * same faction;
/// * level spread within the stage's limit;
/// * item level: compatible when either is 0 or the difference is within the stage's percent
///   of the lower value;
/// * both at least the dungeon's minimum level.
pub fn compatible(a: &Ad, b: &Ad, stage: u8, min_level: u8) -> bool {
    let rules = widen::rules(stage);
    if a.faction != b.faction {
        return false;
    }
    if a.level < min_level || b.level < min_level {
        return false;
    }
    if a.level.abs_diff(b.level) > rules.level_spread {
        return false;
    }
    if a.item_level != 0 && b.item_level != 0 {
        let diff = a.item_level.abs_diff(b.item_level) as u32;
        let low = a.item_level.min(b.item_level) as u32;
        if diff * 100 > rules.item_level_percent * low {
            return false;
        }
    }
    true
}

/// A pair's stage: the stricter of the two.
pub fn pair_stage(a: &Ad, b: &Ad) -> u8 {
    a.stage.min(b.stage)
}

/// Topic a message belongs to is given by the network layer. Helper for turning a verified
/// message into an engine event.
pub fn event_from_message(topic: TopicId, msg: Message) -> super::state::Event {
    use super::state::Event;
    match msg {
        Message::Ad(ad) => Event::AdReceived { topic, ad },
        Message::Taken { peer, counter } => Event::TakenReceived {
            topic,
            peer,
            counter,
        },
        Message::PartyAnnounce {
            peer,
            epoch,
            members,
            party_id,
        } => Event::AnnounceReceived {
            topic,
            from: peer,
            epoch,
            members,
            party_id,
        },
        Message::Ready {
            peer, party_id, ad, ..
        } => Event::ReadyReceived {
            topic,
            from: peer,
            party_id,
            ad,
        },
        Message::Invite { peer, epoch, ad } => Event::InviteReceived {
            topic,
            from: peer,
            epoch,
            ad,
        },
        Message::Accept { peer, epoch } => Event::AcceptReceived {
            topic,
            from: peer,
            epoch,
        },
        Message::Confirm { peer, party_id } => Event::ConfirmReceived {
            topic,
            from: peer,
            party_id,
        },
        Message::Decline { peer, party_id } => Event::DeclineReceived {
            topic,
            from: peer,
            party_id,
        },
        Message::Heartbeat {
            peer,
            party_id,
            nonce,
        } => Event::HeartbeatReceived {
            topic,
            from: peer,
            party_id,
            nonce,
        },
        Message::HeartbeatAck { peer, party_id, .. } => Event::HeartbeatAckReceived {
            from: peer,
            party_id,
        },
    }
}

#[cfg(test)]
pub(crate) fn test_ad(n: u8, role: Role, level: u8, item_level: u16) -> Ad {
    Ad {
        peer: PeerId([n; 32]),
        counter: 1,
        role,
        secondary_role: None,
        level,
        item_level,
        faction: Faction::Alliance,
        stage: 0,
        first_name: format!("P{n}"),
        surname: "Test".into(),
        zone: String::new(),
        created_at: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factions_never_match() {
        let a = test_ad(1, Role::Tank, 24, 100);
        let mut b = test_ad(2, Role::Healer, 24, 100);
        b.faction = Faction::Horde;
        for stage in 0..=4 {
            assert!(!compatible(&a, &b, stage, 1));
        }
    }

    #[test]
    fn level_spread_by_stage_and_symmetry() {
        let a = test_ad(1, Role::Tank, 20, 0);
        let b = test_ad(2, Role::Healer, 23, 0);
        let c = test_ad(3, Role::Damage, 26, 0);
        let d = test_ad(4, Role::Damage, 30, 0);
        assert!(compatible(&a, &b, 0, 1) && compatible(&b, &a, 0, 1)); // 3
        assert!(!compatible(&a, &c, 0, 1) && !compatible(&c, &a, 0, 1)); // 6
        assert!(compatible(&a, &c, 3, 1)); // 6 allowed at stage 3
        assert!(!compatible(&a, &d, 3, 1)); // 10 not at 3
        assert!(compatible(&a, &d, 4, 1) && compatible(&d, &a, 4, 1));
    }

    #[test]
    fn item_level_percent_of_the_lower_value() {
        let a = test_ad(1, Role::Tank, 24, 100);
        let b = test_ad(2, Role::Healer, 24, 115);
        let c = test_ad(3, Role::Healer, 24, 116);
        assert!(compatible(&a, &b, 0, 1)); // exactly 15% of 100
        assert!(!compatible(&a, &c, 0, 1));
        assert!(compatible(&a, &c, 2, 1)); // 30% at stage 2
        let unknown = test_ad(4, Role::Damage, 24, 0);
        assert!(compatible(&a, &unknown, 0, 1)); // 0 is unknown and always fits
    }

    #[test]
    fn minimum_level_applies_to_both() {
        let a = test_ad(1, Role::Tank, 14, 0);
        let b = test_ad(2, Role::Healer, 16, 0);
        assert!(compatible(&a, &b, 0, 14));
        assert!(!compatible(&a, &b, 0, 15));
    }

    #[test]
    fn counter_wraparound() {
        assert!(counter_newer(2, 1));
        assert!(!counter_newer(1, 2));
        assert!(!counter_newer(5, 5));
        assert!(counter_newer(0, 65535));
    }
}
