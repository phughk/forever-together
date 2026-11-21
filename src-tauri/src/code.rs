//! Character code decoder (spec section 1). Decoding is in the app. The encoder is test-only,
//! because the addon is the real encoder.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PREFIX: &str = "WFT1:";
const BASE45: &[u8; 45] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
/// Fixed part (16 bytes) + realm, first name, surname (1 length byte + at least 1 byte each)
/// + zone length byte + check (2 bytes).
const MIN_RECORD: usize = 16 + 2 + 2 + 2 + 1 + 2;
pub const MAX_NAME: usize = 24;
pub const MAX_ZONE: usize = 32;

/// One vocabulary on both sides (spec "Errors").
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CodeError {
    #[error("unknown version: update the addon or the app")]
    UnknownVersion,
    #[error("the check does not match: the text was mistyped or cut")]
    BadCheck,
    #[error("the code is too short")]
    Truncated,
    #[error("the text is not a valid code")]
    BadText,
    #[error("unexpected bytes after the record")]
    TrailingBytes,
    #[error("invalid field: {0}")]
    InvalidField(&'static str),
}

impl CodeError {
    /// Name as used in the test vectors and in the addon.
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnknownVersion => "UnknownVersion",
            Self::BadCheck => "BadCheck",
            Self::Truncated => "Truncated",
            Self::BadText => "BadText",
            Self::TrailingBytes => "TrailingBytes",
            Self::InvalidField(_) => "InvalidField",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    Tank,
    Healer,
    Damage,
}

impl Role {
    fn from_bits(b: u8) -> Option<Role> {
        match b {
            1 => Some(Role::Tank),
            2 => Some(Role::Healer),
            3 => Some(Role::Damage),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Faction {
    Alliance,
    Horde,
}

impl Faction {
    /// The text used in topic strings.
    pub fn slug(self) -> &'static str {
        match self {
            Faction::Alliance => "alliance",
            Faction::Horde => "horde",
        }
    }
}

/// What the app understood from a code. Every field is a claim by the sender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    /// `None` when the client could not tell (role bits 0).
    pub role: Option<Role>,
    pub secondary_role: Option<Role>,
    pub faction: Faction,
    pub class: u8,
    pub race: u8,
    pub level: u8,
    /// 0 when unknown.
    pub item_level: u16,
    pub spec: u16,
    /// Epoch minutes (UTC).
    pub created_minutes: u32,
    pub counter: u16,
    pub region: u8,
    pub realm: String,
    pub first_name: String,
    pub surname: String,
    pub zone: String,
}

pub fn base45_decode(text: &str) -> Result<Vec<u8>, CodeError> {
    let mut vals = Vec::with_capacity(text.len());
    for b in text.bytes() {
        vals.push(
            BASE45
                .iter()
                .position(|&c| c == b)
                .ok_or(CodeError::BadText)? as u32,
        );
    }
    let mut out = Vec::with_capacity(vals.len() / 3 * 2 + 2);
    for chunk in vals.chunks(3) {
        match chunk {
            [c, d, e] => {
                let n = c + d * 45 + e * 45 * 45;
                if n > 0xFFFF {
                    return Err(CodeError::BadText);
                }
                out.extend_from_slice(&(n as u16).to_be_bytes());
            }
            [c, d] => {
                let n = c + d * 45;
                if n > 0xFF {
                    return Err(CodeError::BadText);
                }
                out.push(n as u8);
            }
            _ => return Err(CodeError::BadText),
        }
    }
    Ok(out)
}

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF, no reflection, no final xor).
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], CodeError> {
        let end = self.at.checked_add(n).ok_or(CodeError::Truncated)?;
        let s = self.b.get(self.at..end).ok_or(CodeError::Truncated)?;
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, CodeError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, CodeError> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes([s[0], s[1]]))
    }
    fn string(&mut self, field: &'static str, min: usize, max: usize) -> Result<String, CodeError> {
        let n = self.u8()? as usize;
        if n < min || n > max {
            return Err(CodeError::InvalidField(field));
        }
        let s = self.take(n)?;
        String::from_utf8(s.to_vec()).map_err(|_| CodeError::InvalidField(field))
    }
}

/// Decode pasted text. Strips leading and trailing whitespace only: a space is a base45
/// character and must survive inside the text.
pub fn decode(text: &str) -> Result<Character, CodeError> {
    let text = text.trim();
    let body = match text.strip_prefix(PREFIX) {
        Some(b) => b,
        None => {
            // "WFT<digit>:" with another digit is a future version.
            let b = text.as_bytes();
            if b.len() >= 5 && &b[..3] == b"WFT" && b[3].is_ascii_digit() && b[4] == b':' {
                return Err(CodeError::UnknownVersion);
            }
            return Err(CodeError::BadText);
        }
    };
    let rec = base45_decode(body)?;
    decode_record(&rec)
}

pub fn decode_record(rec: &[u8]) -> Result<Character, CodeError> {
    match rec.first() {
        None => return Err(CodeError::Truncated),
        Some(1) => {}
        Some(_) => return Err(CodeError::UnknownVersion),
    }
    if rec.len() < MIN_RECORD {
        return Err(CodeError::Truncated);
    }
    let (data, check) = rec.split_at(rec.len() - 2);
    if crc16(data) != u16::from_be_bytes([check[0], check[1]]) {
        return Err(CodeError::BadCheck);
    }
    let mut r = Reader { b: data, at: 1 };
    let flags = r.u8()?;
    if flags & 0xC0 != 0 {
        return Err(CodeError::InvalidField("flags"));
    }
    let class = r.u8()?;
    let race = r.u8()?;
    let level = r.u8()?;
    if level == 0 {
        return Err(CodeError::InvalidField("level"));
    }
    let item_level = r.u16()?;
    let spec = r.u16()?;
    let created = r.take(4)?;
    let created_minutes = u32::from_be_bytes([created[0], created[1], created[2], created[3]]);
    let counter = r.u16()?;
    let region = r.u8()?;
    let realm = r.string("realm", 1, MAX_NAME)?;
    let first_name = r.string("firstName", 1, MAX_NAME)?;
    let surname = r.string("surname", 1, MAX_NAME)?;
    let zone = r.string("zone", 0, MAX_ZONE)?;
    if r.at != data.len() {
        return Err(CodeError::TrailingBytes);
    }
    Ok(Character {
        role: Role::from_bits(flags & 3),
        secondary_role: Role::from_bits((flags >> 2) & 3),
        faction: if flags & 0x10 == 0 {
            Faction::Alliance
        } else {
            Faction::Horde
        },
        class,
        race,
        level,
        item_level: if flags & 0x20 != 0 { item_level } else { 0 },
        spec,
        created_minutes,
        counter,
        region,
        realm,
        first_name,
        surname,
        zone,
    })
}

/// Test-only encoder, the reference for round trips and for building fixtures elsewhere.
#[cfg(test)]
pub mod test_encoder {
    use super::*;

    pub fn base45_encode(data: &[u8]) -> String {
        let mut s = String::new();
        for chunk in data.chunks(2) {
            if let [a, b] = chunk {
                let n = (*a as u32) * 256 + *b as u32;
                for d in [n % 45, (n / 45) % 45, n / (45 * 45)] {
                    s.push(BASE45[d as usize] as char);
                }
            } else {
                let n = chunk[0] as u32;
                s.push(BASE45[(n % 45) as usize] as char);
                s.push(BASE45[(n / 45) as usize] as char);
            }
        }
        s
    }

    pub fn encode_record(c: &Character) -> Vec<u8> {
        let mut flags = 0u8;
        flags |= match c.role {
            None => 0,
            Some(Role::Tank) => 1,
            Some(Role::Healer) => 2,
            Some(Role::Damage) => 3,
        };
        flags |= match c.secondary_role {
            None => 0,
            Some(Role::Tank) => 1 << 2,
            Some(Role::Healer) => 2 << 2,
            Some(Role::Damage) => 3 << 2,
        };
        if c.faction == Faction::Horde {
            flags |= 0x10;
        }
        if c.item_level != 0 {
            flags |= 0x20;
        }
        let mut v = vec![1, flags, c.class, c.race, c.level];
        v.extend_from_slice(&c.item_level.to_be_bytes());
        v.extend_from_slice(&c.spec.to_be_bytes());
        v.extend_from_slice(&c.created_minutes.to_be_bytes());
        v.extend_from_slice(&c.counter.to_be_bytes());
        v.push(c.region);
        for s in [&c.realm, &c.first_name, &c.surname, &c.zone] {
            v.push(s.len() as u8);
            v.extend_from_slice(s.as_bytes());
        }
        let crc = crc16(&v);
        v.extend_from_slice(&crc.to_be_bytes());
        v
    }

    pub fn encode(c: &Character) -> String {
        format!("{PREFIX}{}", base45_encode(&encode_record(c)))
    }

    pub fn sample_character() -> Character {
        Character {
            role: Some(Role::Tank),
            secondary_role: None,
            faction: Faction::Alliance,
            class: 1,
            race: 1,
            level: 24,
            item_level: 120,
            spec: 73,
            created_minutes: 29_000_000,
            counter: 3,
            region: 3,
            realm: "5".into(),
            first_name: "Aldric".into(),
            surname: "Thornwood".into(),
            zone: "Westfall".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_encoder::*;
    use super::*;

    #[test]
    fn base45_rfc9285_examples() {
        assert_eq!(base45_encode(b"AB"), "BB8");
        assert_eq!(base45_encode(b"Hello!!"), "%69 VD92EX0");
        assert_eq!(base45_encode(b"base-45"), "UJCLQE7W581");
        assert_eq!(base45_encode(b"ietf!"), "QED8WEX0");
        assert_eq!(base45_decode("QED8WEX0").unwrap(), b"ietf!");
        assert_eq!(base45_decode("%69 VD92EX0").unwrap(), b"Hello!!");
    }

    #[test]
    fn base45_odd_and_even_lengths_round_trip() {
        for len in 0..12u8 {
            let data: Vec<u8> = (0..len)
                .map(|i| i.wrapping_mul(37).wrapping_add(250))
                .collect();
            assert_eq!(base45_decode(&base45_encode(&data)).unwrap(), data);
        }
    }

    #[test]
    fn crc_check_value() {
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn round_trip() {
        let c = sample_character();
        assert_eq!(decode(&encode(&c)).unwrap(), c);
    }

    #[test]
    fn accented_and_maximum_strings() {
        let mut c = sample_character();
        c.first_name = "Éowyn".into();
        c.surname = "Ünterwald".into();
        c.zone = "z".repeat(MAX_ZONE);
        c.realm = "r".repeat(MAX_NAME);
        assert_eq!(decode(&encode(&c)).unwrap(), c);
    }

    #[test]
    fn unknown_item_level_and_roles() {
        let mut c = sample_character();
        c.item_level = 0;
        c.role = None;
        c.faction = Faction::Horde;
        c.zone = String::new();
        let d = decode(&encode(&c)).unwrap();
        assert_eq!(d, c);
        assert_eq!(d.item_level, 0);
    }

    #[test]
    fn surrounding_whitespace_is_stripped() {
        let c = sample_character();
        assert_eq!(decode(&format!("  \n{}\r\n", encode(&c))).unwrap(), c);
    }

    #[test]
    fn internal_space_survives() {
        // A space is a base45 character. Find a record whose text has one and prove that
        // stripping it would break the code while the decoder accepts it.
        let mut c = sample_character();
        let mut text = encode(&c);
        for n in 0..2000u16 {
            if text.contains(' ') {
                break;
            }
            c.counter = n;
            text = encode(&c);
        }
        assert!(text.contains(' '), "no sample with an inner space found");
        assert_eq!(decode(&text).unwrap(), c);
        assert!(decode(&text.replace(' ', "")).is_err());
    }

    #[test]
    fn errors() {
        let c = sample_character();
        let good = encode(&c);
        // bad check
        let rec = encode_record(&c);
        let mut bad = rec.clone();
        bad[10] ^= 1;
        assert_eq!(decode_record(&bad), Err(CodeError::BadCheck));
        // truncated record
        assert_eq!(decode_record(&rec[..10]), Err(CodeError::Truncated));
        assert_eq!(decode_record(&[]), Err(CodeError::Truncated));
        // truncated text (cut in the middle) fails the check
        assert!(decode(&good[..good.len() - 4]).is_err());
        // unknown version
        let mut v2 = rec.clone();
        v2[0] = 2;
        assert_eq!(
            decode(&format!("{PREFIX}{}", base45_encode(&v2))),
            Err(CodeError::UnknownVersion)
        );
        assert_eq!(decode("WFT2:ABC"), Err(CodeError::UnknownVersion));
        // bad text
        assert_eq!(decode("hello"), Err(CodeError::BadText));
        assert_eq!(decode("WFT1:abc"), Err(CodeError::BadText));
        assert_eq!(decode("WFT1:ZZZ"), Err(CodeError::BadText)); // overflows 16 bits
        assert_eq!(decode("WFT1:A"), Err(CodeError::BadText)); // dangling single char
        // trailing bytes (valid check over a longer body)
        let mut long = rec[..rec.len() - 2].to_vec();
        long.push(0);
        let crc = crc16(&long);
        long.extend_from_slice(&crc.to_be_bytes());
        assert_eq!(decode_record(&long), Err(CodeError::TrailingBytes));
        // invalid fields
        let mut zero_level = sample_character();
        zero_level.level = 0;
        assert_eq!(
            decode(&encode(&zero_level)),
            Err(CodeError::InvalidField("level"))
        );
        let mut empty = sample_character();
        empty.surname = String::new();
        assert_eq!(
            decode(&encode(&empty)),
            Err(CodeError::InvalidField("surname"))
        );
        let mut long_name = sample_character();
        long_name.first_name = "x".repeat(MAX_NAME + 1);
        assert_eq!(
            decode(&encode(&long_name)),
            Err(CodeError::InvalidField("firstName"))
        );
        let mut reserved = rec[..rec.len() - 2].to_vec();
        reserved[1] |= 0x40;
        let crc = crc16(&reserved);
        reserved.extend_from_slice(&crc.to_be_bytes());
        assert_eq!(
            decode_record(&reserved),
            Err(CodeError::InvalidField("flags"))
        );
    }

    #[test]
    fn counter_wraparound_fields() {
        let mut c = sample_character();
        c.counter = 65535;
        assert_eq!(decode(&encode(&c)).unwrap().counter, 65535);
        c.counter = 0;
        assert_eq!(decode(&encode(&c)).unwrap().counter, 0);
    }

    #[test]
    fn typical_record_fits_the_target() {
        assert!(encode(&sample_character()).len() <= 122);
    }

    /// Shared vectors in `docs/rdf-code-vectors.json` (created by plan step 1). Skipped with a
    /// message while the file is absent. The assumed shape, until the file exists: an object
    /// with `cases`, each `{ name, text, error? }` where `error` is an error name and a case
    /// without `error` must decode.
    #[test]
    fn shared_vectors() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/rdf-code-vectors.json");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            eprintln!(
                "SKIPPED shared_vectors: {} does not exist yet (plan step 1)",
                path.display()
            );
            return;
        };
        let v: serde_json::Value = serde_json::from_str(&raw).expect("vectors file is JSON");
        let cases = v
            .get("cases")
            .and_then(|c| c.as_array())
            .or_else(|| v.as_array());
        let Some(cases) = cases else {
            eprintln!("SKIPPED shared_vectors: file has no recognised `cases` array");
            return;
        };
        for case in cases {
            let Some(text) = case.get("text").and_then(|t| t.as_str()) else {
                continue;
            };
            let name = case.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let expected = case
                .get("error")
                .or_else(|| case.get("err"))
                .and_then(|e| e.as_str());
            match (expected, decode(text)) {
                (None, Ok(_)) => {}
                (Some(e), Err(got)) => {
                    assert_eq!(got.name(), e.split('(').next().unwrap(), "case {name}")
                }
                (None, Err(got)) => panic!("case {name}: expected success, got {got:?}"),
                (Some(e), Ok(_)) => panic!("case {name}: expected {e}, decoded fine"),
            }
        }
    }
}
