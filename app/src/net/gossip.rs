//! The signed gossip envelope (spec section 3, ADR 0019).
//!
//! iroh-gossip 0.103 has no author field and does not sign content: `Message::delivered_from`
//! is "not the same as the original author" and its docs say "sign it in your app". So every
//! message is sealed here and opened here before an event may reach the engine.
//!
//! Wire: `postcard(Envelope { author, sig, payload })` with `payload = postcard(Payload { to, msg })`
//! and `sig = ed25519(topic || 0x01 || payload)`. The topic and the version byte inside the
//! signed bytes stop a replay into another swarm or protocol version. `to` is `Some` for a
//! message meant for one peer (gossip has no direct send, see `iroh_net`): others drop it
//! after verifying. The author inside the message (`peer`) must equal the envelope author, so a
//! member cannot sign a message that speaks for another.

use crate::dungeons::TopicId;
use crate::engine::{Message, NetError};
use crate::identity::{PeerId, Signer, verify};
use serde::{Deserialize, Serialize};

/// Domain byte after the topic in the signed bytes (protocol version 1).
pub const SIGN_DOMAIN: u8 = 0x01;
/// Upper bound for a sealed message. iroh-gossip's own default maximum is larger.
pub const MAX_ENVELOPE: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Envelope {
    author: PeerId,
    /// 64 bytes. A `Vec` because serde has no impl for arrays of 64.
    sig: Vec<u8>,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Payload {
    to: Option<PeerId>,
    msg: Message,
}

/// A message whose author is proven.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    pub author: PeerId,
    pub to: Option<PeerId>,
    pub msg: Message,
}

impl Verified {
    /// True when the message is for `me`: a broadcast, or addressed to me.
    pub fn is_for(&self, me: PeerId) -> bool {
        self.to.is_none_or(|t| t == me)
    }
}

/// The peer a message says it is from.
pub fn author_of(msg: &Message) -> PeerId {
    match msg {
        Message::Ad(a) => a.peer,
        Message::Taken { peer, .. }
        | Message::PartyAnnounce { peer, .. }
        | Message::Ready { peer, .. }
        | Message::Confirm { peer, .. }
        | Message::Decline { peer, .. }
        | Message::Heartbeat { peer, .. }
        | Message::HeartbeatAck { peer, .. }
        | Message::Invite { peer, .. }
        | Message::Accept { peer, .. }
        | Message::NewMember { peer, .. } => *peer,
    }
}

fn signed_bytes(topic: TopicId, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(33 + payload.len());
    v.extend_from_slice(&topic.0);
    v.push(SIGN_DOMAIN);
    v.extend_from_slice(payload);
    v
}

/// Sign and serialize `msg` for `topic`. The signer must be the message's author.
pub fn seal(
    signer: &dyn Signer,
    topic: TopicId,
    to: Option<PeerId>,
    msg: &Message,
) -> Result<Vec<u8>, NetError> {
    if author_of(msg) != signer.peer_id() {
        return Err(NetError::BadSignature);
    }
    let payload = postcard::to_stdvec(&Payload {
        to,
        msg: msg.clone(),
    })
    .map_err(|e| NetError::Io(format!("encode: {e}")))?;
    let sig = signer.sign(&signed_bytes(topic, &payload)).to_vec();
    let bytes = postcard::to_stdvec(&Envelope {
        author: signer.peer_id(),
        sig,
        payload,
    })
    .map_err(|e| NetError::Io(format!("encode: {e}")))?;
    if bytes.len() > MAX_ENVELOPE {
        return Err(NetError::Io("message too large".into()));
    }
    Ok(bytes)
}

/// Verify and parse bytes received on `topic`. `BadSignature` for a wrong signature, a wrong
/// topic, or an author that differs from the message's own `peer`; `Io` for bytes that are not
/// an envelope at all. Nothing is returned to the engine unless this succeeds.
pub fn open(topic: TopicId, bytes: &[u8]) -> Result<Verified, NetError> {
    if bytes.len() > MAX_ENVELOPE {
        return Err(NetError::Io("message too large".into()));
    }
    let env: Envelope =
        postcard::from_bytes(bytes).map_err(|_| NetError::Io("malformed envelope".into()))?;
    let sig: [u8; 64] = env
        .sig
        .as_slice()
        .try_into()
        .map_err(|_| NetError::BadSignature)?;
    if !verify(&env.author, &signed_bytes(topic, &env.payload), &sig) {
        return Err(NetError::BadSignature);
    }
    let p: Payload =
        postcard::from_bytes(&env.payload).map_err(|_| NetError::Io("malformed payload".into()))?;
    if author_of(&p.msg) != env.author {
        return Err(NetError::BadSignature);
    }
    Ok(Verified {
        author: env.author,
        to: p.to,
        msg: p.msg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::PartyId;
    use crate::identity::Identity;

    const T: TopicId = TopicId([7; 32]);

    fn msg(peer: PeerId) -> Message {
        Message::Confirm {
            peer,
            party_id: PartyId([1; 32]),
        }
    }

    #[test]
    fn round_trip() {
        let id = Identity::generate();
        let m = msg(id.peer_id());
        let bytes = seal(&id, T, None, &m).unwrap();
        let v = open(T, &bytes).unwrap();
        assert_eq!(v.author, id.peer_id());
        assert_eq!(v.msg, m);
        assert!(v.is_for(PeerId([9; 32])));
    }

    #[test]
    fn rejects_a_flipped_bit_anywhere() {
        let id = Identity::generate();
        let bytes = seal(&id, T, None, &msg(id.peer_id())).unwrap();
        let mut rejected = 0;
        for i in 0..bytes.len() {
            let mut b = bytes.clone();
            b[i] ^= 0x01;
            // A flipped bit either fails to parse or fails the signature. It never opens as
            // the same message.
            match open(T, &b) {
                Ok(v) => assert_ne!(v.msg, msg(id.peer_id()), "byte {i}"),
                Err(_) => rejected += 1,
            }
        }
        assert!(rejected > bytes.len() / 2);
    }

    #[test]
    fn rejects_a_replay_into_another_topic() {
        let id = Identity::generate();
        let bytes = seal(&id, T, None, &msg(id.peer_id())).unwrap();
        assert_eq!(open(TopicId([8; 32]), &bytes), Err(NetError::BadSignature));
    }

    #[test]
    fn rejects_a_message_signed_by_someone_else() {
        // Forger signs a message that claims to be from the victim, and puts its own key as
        // the author: the signature is valid but the message's author differs.
        let victim = Identity::generate();
        let forger = Identity::generate();
        let payload = postcard::to_stdvec(&Payload {
            to: None,
            msg: msg(victim.peer_id()),
        })
        .unwrap();
        let sig = forger.sign(&signed_bytes(T, &payload)).to_vec();
        let bytes = postcard::to_stdvec(&Envelope {
            author: forger.peer_id(),
            sig,
            payload: payload.clone(),
        })
        .unwrap();
        assert_eq!(open(T, &bytes), Err(NetError::BadSignature));
        // Putting the victim's key as author with the forger's signature fails the check.
        let sig = forger.sign(&signed_bytes(T, &payload)).to_vec();
        let bytes = postcard::to_stdvec(&Envelope {
            author: victim.peer_id(),
            sig,
            payload,
        })
        .unwrap();
        assert_eq!(open(T, &bytes), Err(NetError::BadSignature));
    }

    #[test]
    fn seal_refuses_to_speak_for_another_peer() {
        let a = Identity::generate();
        let b = Identity::generate();
        assert_eq!(
            seal(&a, T, None, &msg(b.peer_id())),
            Err(NetError::BadSignature)
        );
    }

    #[test]
    fn short_signature_and_garbage_are_rejected() {
        let id = Identity::generate();
        let payload = postcard::to_stdvec(&Payload {
            to: None,
            msg: msg(id.peer_id()),
        })
        .unwrap();
        let bytes = postcard::to_stdvec(&Envelope {
            author: id.peer_id(),
            sig: vec![0; 10],
            payload,
        })
        .unwrap();
        assert_eq!(open(T, &bytes), Err(NetError::BadSignature));
        assert!(matches!(open(T, b"garbage"), Err(NetError::Io(_))));
        assert!(matches!(open(T, &[]), Err(NetError::Io(_))));
        assert!(matches!(
            open(T, &vec![0u8; MAX_ENVELOPE + 1]),
            Err(NetError::Io(_))
        ));
    }

    #[test]
    fn addressed_message_is_only_for_its_target() {
        let a = Identity::generate();
        let target = PeerId([5; 32]);
        let bytes = seal(&a, T, Some(target), &msg(a.peer_id())).unwrap();
        let v = open(T, &bytes).unwrap();
        assert!(v.is_for(target));
        assert!(!v.is_for(PeerId([6; 32])));
        // The target cannot be changed without breaking the signature.
        let mut env: Envelope = postcard::from_bytes(&bytes).unwrap();
        let mut p: Payload = postcard::from_bytes(&env.payload).unwrap();
        p.to = None;
        env.payload = postcard::to_stdvec(&p).unwrap();
        assert_eq!(
            open(T, &postcard::to_stdvec(&env).unwrap()),
            Err(NetError::BadSignature)
        );
    }
}
