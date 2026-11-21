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
    // Another prefix is `BadText` (spec "Errors"): a future version is reported from the version
    // byte of a record with the right prefix, never from the prefix text.
    let body = text.strip_prefix(PREFIX).ok_or(CodeError::BadText)?;
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

/// Display name of a region number. Display only: topics hash the raw number. Verified live 2026-11-21: 3 = EU. Assumed
/// (standard `GetCurrentRegion` numbering, not yet seen): 1 US, 2 KR, 4 TW, 5 CN. Same table as `addon/ForeverTogether/Names.lua`.
pub fn region_name(region: u8) -> String {
    match region {
        0 => "unknown".to_owned(),
        1 => "US".to_owned(),
        2 => "KR".to_owned(),
        3 => "EU".to_owned(),
        4 => "TW".to_owned(),
        5 => "CN".to_owned(),
        n => format!("region {n}"),
    }
}

/// Display name of a realm value (the game-mode record ID as a decimal string). Display only: topics hash the raw value.
/// Verified live 2026-11-21: 14 = PvP, 15 = PvE, 16 = RP-PvE, 18 = RP-PvP. The Hardcore ID is unknown (not released) and not guessed.
pub fn realm_name(realm: &str) -> String {
    match realm {
        "14" => "PvP".to_owned(),
        "15" => "PvE".to_owned(),
        "16" => "RP-PvE".to_owned(),
        "18" => "RP-PvP".to_owned(),
        other => format!("realm {other}"),
    }
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
        // A wrong prefix is BadText; a future version shows in the version byte (vectors).
        assert_eq!(decode("WFT2:ABC"), Err(CodeError::BadText));
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

    // ---- Shared vectors: docs/rdf-code-vectors.json (also read by the addon tests) ----

    use serde_json::Value;

    fn vectors() -> Value {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/rdf-code-vectors.json");
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        serde_json::from_str(&raw).expect("vectors file is JSON")
    }

    fn list<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
        let a = v[key]
            .as_array()
            .unwrap_or_else(|| panic!("`{key}` is not an array"));
        assert!(!a.is_empty(), "`{key}` is empty");
        a
    }

    fn s<'a>(v: &'a Value, key: &str) -> &'a str {
        v[key]
            .as_str()
            .unwrap_or_else(|| panic!("`{key}` missing in {v}"))
    }

    fn hex(h: &str) -> Vec<u8> {
        data_encoding::HEXLOWER.decode(h.as_bytes()).unwrap()
    }

    /// The vocabulary of the vectors: `Name` or `InvalidField(field)`.
    fn describe(e: &CodeError) -> String {
        match e {
            CodeError::InvalidField(f) => format!("InvalidField({f})"),
            other => other.name().to_owned(),
        }
    }

    fn int(f: &Value, key: &'static str, max: u64) -> Result<u64, CodeError> {
        let n = f[key].as_u64().unwrap_or(u64::MAX);
        if n > max {
            return Err(CodeError::InvalidField(key));
        }
        Ok(n)
    }

    fn text_field(
        f: &Value,
        key: &'static str,
        min: usize,
        max: usize,
    ) -> Result<String, CodeError> {
        let t = f[key].as_str().ok_or(CodeError::InvalidField(key))?;
        if t.len() < min || t.len() > max {
            return Err(CodeError::InvalidField(key));
        }
        Ok(t.to_owned())
    }

    /// What an encoder must accept: vector `fields` to a `Character`, or the encode error.
    fn character_from_fields(f: &Value) -> Result<Character, CodeError> {
        let faction = match f["faction"].as_str() {
            Some("alliance") => Faction::Alliance,
            Some("horde") => Faction::Horde,
            _ => return Err(CodeError::InvalidField("faction")),
        };
        let level = int(f, "level", 255)?;
        if level == 0 {
            return Err(CodeError::InvalidField("level"));
        }
        Ok(Character {
            role: Role::from_bits(int(f, "role", 3)? as u8),
            secondary_role: Role::from_bits(int(f, "secondaryRole", 3)? as u8),
            faction,
            class: int(f, "class", 255)? as u8,
            race: int(f, "race", 255)? as u8,
            level: level as u8,
            item_level: int(f, "itemLevel", 65535)? as u16,
            spec: int(f, "spec", 65535)? as u16,
            created_minutes: int(f, "created", u32::MAX as u64)? as u32,
            counter: int(f, "counter", 65535)? as u16,
            region: int(f, "region", 255)? as u8,
            realm: text_field(f, "realm", 1, MAX_NAME)?,
            first_name: text_field(f, "firstName", 1, MAX_NAME)?,
            surname: text_field(f, "surname", 1, MAX_NAME)?,
            zone: text_field(f, "zone", 0, MAX_ZONE)?,
        })
    }

    /// Every category of the file is exercised: a category the tests do not know fails.
    #[test]
    fn shared_vectors_cover_every_category() {
        let v = vectors();
        let handled = [
            "_comment",
            "prefix",
            "base45",
            "base45_errors",
            "crc16",
            "records",
            "decode_errors",
            "paste",
            "encode_errors",
            "counter_wrap",
            "limits",
            "qr",
            "qr_capacity_alnum_m",
            "qr_errors",
        ];
        for k in v.as_object().unwrap().keys() {
            assert!(
                handled.contains(&k.as_str()),
                "vector category `{k}` is not tested"
            );
        }
        for k in handled {
            assert!(v.get(k).is_some(), "vector category `{k}` disappeared");
        }
        assert_eq!(s(&v, "prefix"), PREFIX);
    }

    #[test]
    fn region_and_realm_display_names() {
        assert_eq!(region_name(3), "EU");
        assert_eq!(region_name(0), "unknown");
        assert_eq!(region_name(9), "region 9");
        assert_eq!(realm_name("14"), "PvP");
        assert_eq!(realm_name("15"), "PvE");
        assert_eq!(realm_name("16"), "RP-PvE");
        assert_eq!(realm_name("18"), "RP-PvP");
        assert_eq!(realm_name("27"), "realm 27");
    }

    #[test]
    fn synthetic_realm_vectors_decode_to_known_names() {
        let v = vectors();
        let mut n = 0;
        for c in list(&v, "records") {
            if !s(c, "name").starts_with("synthetic:") {
                continue;
            }
            let ch = decode(s(c, "text")).unwrap();
            assert_eq!(region_name(ch.region), "EU");
            assert!(["PvE", "PvP", "RP-PvP", "realm 27"].contains(&realm_name(&ch.realm).as_str()));
            n += 1;
        }
        assert_eq!(n, 4);
    }

    #[test]
    fn shared_vectors_base45() {
        let v = vectors();
        for c in list(&v, "base45") {
            let name = s(c, "name");
            assert_eq!(
                base45_decode(s(c, "text")).unwrap(),
                hex(s(c, "hex")),
                "{name}"
            );
            assert_eq!(base45_encode(&hex(s(c, "hex"))), s(c, "text"), "{name}");
        }
        for c in list(&v, "base45_errors") {
            assert_eq!(
                base45_decode(s(c, "text")),
                Err(CodeError::BadText),
                "{}",
                s(c, "name")
            );
        }
    }

    #[test]
    fn shared_vectors_crc16() {
        for c in list(&vectors(), "crc16") {
            assert_eq!(
                crc16(&hex(s(c, "hex"))) as u64,
                c["crc"].as_u64().unwrap(),
                "{}",
                s(c, "name")
            );
        }
    }

    #[test]
    fn shared_vectors_records_decode_and_encode() {
        let v = vectors();
        let mut min = usize::MAX;
        let mut max = 0;
        for c in list(&v, "records") {
            let name = s(c, "name");
            let want =
                character_from_fields(&c["fields"]).unwrap_or_else(|e| panic!("{name}: {e}"));
            let bytes = hex(s(c, "hex"));
            min = min.min(bytes.len());
            max = max.max(bytes.len());
            assert_eq!(decode(s(c, "text")).as_ref(), Ok(&want), "decode {name}");
            assert_eq!(
                decode_record(&bytes).as_ref(),
                Ok(&want),
                "decode_record {name}"
            );
            assert_eq!(encode_record(&want), bytes, "encode_record {name}");
            assert_eq!(encode(&want), s(c, "text"), "encode {name}");
        }
        let lim = &v["limits"];
        assert_eq!(MIN_RECORD as u64, lim["min_record_bytes"].as_u64().unwrap());
        assert_eq!(min as u64, lim["min_record_bytes"].as_u64().unwrap());
        assert_eq!(max as u64, lim["max_record_bytes"].as_u64().unwrap());
    }

    #[test]
    fn shared_vectors_decode_errors() {
        let mut seen = std::collections::BTreeSet::new();
        for c in list(&vectors(), "decode_errors") {
            let name = s(c, "name");
            let want = s(c, "error");
            match decode(s(c, "text")) {
                Err(e) => assert_eq!(describe(&e), want, "{name}"),
                Ok(_) => panic!("{name}: expected {want}, decoded fine"),
            }
            seen.insert(want.split('(').next().unwrap().to_owned());
        }
        // The whole vocabulary is present in the vectors.
        for n in [
            "UnknownVersion",
            "BadCheck",
            "Truncated",
            "BadText",
            "TrailingBytes",
            "InvalidField",
        ] {
            assert!(seen.contains(n), "no decode_errors vector for {n}");
        }
    }

    #[test]
    fn shared_vectors_paste() {
        for c in list(&vectors(), "paste") {
            let name = s(c, "name");
            let clean = decode(s(c, "text")).unwrap();
            assert_eq!(decode(s(c, "pasted")), Ok(clean), "{name}");
        }
    }

    #[test]
    fn shared_vectors_encode_errors() {
        for c in list(&vectors(), "encode_errors") {
            let name = s(c, "name");
            match character_from_fields(&c["fields"]) {
                Err(e) => assert_eq!(describe(&e), s(c, "error"), "{name}"),
                Ok(_) => panic!("{name}: expected {}, accepted", s(c, "error")),
            }
        }
    }

    #[test]
    fn shared_vectors_counter_wrap() {
        let v = vectors();
        let w = &v["counter_wrap"];
        let (from, to) = (
            w["from"].as_u64().unwrap() as u16,
            w["to"].as_u64().unwrap() as u16,
        );
        // Both ends survive the code, and the engine treats the wrapped counter as newer.
        let mut c = sample_character();
        c.counter = from;
        assert_eq!(decode(&encode(&c)).unwrap().counter, from);
        c.counter = to;
        assert_eq!(decode(&encode(&c)).unwrap().counter, to);
        assert!(crate::engine::ad::counter_newer(to, from));
        assert!(!crate::engine::ad::counter_newer(from, to));
    }

    /// The app does not draw the character QR (the addon does), so the Rust side checks what it
    /// owns: the sizes the vectors promise, the capacity table, and that each matrix is a
    /// well-formed square for the smallest version that fits its text.
    #[test]
    fn shared_vectors_qr_shape_and_capacity() {
        let v = vectors();
        let cap_table = v["qr_capacity_alnum_m"].as_object().unwrap();
        let cap = |ver: u64| cap_table[&ver.to_string()].as_u64().unwrap() as usize;
        assert_eq!(cap_table.len(), 10);
        for (ver, want) in (1..=10).zip([20, 38, 61, 90, 122, 154, 178, 221, 262, 311]) {
            assert_eq!(cap(ver), want, "capacity of version {ver}");
        }
        assert!(encode(&sample_character()).len() <= cap(5));
        for c in list(&v, "records") {
            assert!(s(c, "text").len() <= cap(8), "{}", s(c, "name"));
        }
        for c in list(&v, "qr") {
            let name = s(c, "name");
            let ver = c["version"].as_u64().unwrap();
            let size = c["size"].as_u64().unwrap() as usize;
            let text = s(c, "text");
            assert_eq!(size as u64, 17 + 4 * ver, "{name}");
            assert!(text.len() <= cap(ver), "{name}: text does not fit");
            assert!(
                ver == 1 || text.len() > cap(ver - 1),
                "{name}: not the smallest version"
            );
            assert!(c["mask"].as_u64().unwrap() < 8, "{name}");
            let rows = c["rows"].as_array().unwrap();
            assert_eq!(rows.len(), size, "{name}");
            for r in rows {
                let r = r.as_str().unwrap();
                assert_eq!(r.len(), size, "{name}");
                assert!(r.bytes().all(|b| b == b'0' || b == b'1'), "{name}");
            }
            // Finder patterns: the 7x7 ring and 3x3 core at three corners, and the dark module.
            let at = |r: usize, c: usize| rows[r].as_str().unwrap().as_bytes()[c] == b'1';
            for (r0, c0) in [(0, 0), (0, size - 7), (size - 7, 0)] {
                for i in 0..7 {
                    for j in 0..7 {
                        let ring = i == 0 || i == 6 || j == 0 || j == 6;
                        let core = (2..=4).contains(&i) && (2..=4).contains(&j);
                        assert_eq!(at(r0 + i, c0 + j), ring || core, "{name} finder");
                    }
                }
            }
            assert!(at(4 * ver as usize + 9, 8), "{name}: dark module");
        }
    }

    #[test]
    fn shared_vectors_qr_errors() {
        let v = vectors();
        let max = v["qr_capacity_alnum_m"]["10"].as_u64().unwrap() as usize;
        for c in list(&v, "qr_errors") {
            let name = s(c, "name");
            let text = s(c, "text");
            match s(c, "error") {
                // Valid alphanumeric text longer than version 10 holds.
                "TooLong" => {
                    assert!(text.len() > max, "{name}");
                    assert!(text.bytes().all(|b| BASE45.contains(&b)), "{name}");
                }
                // Not encodable in alphanumeric mode: empty, or characters outside it.
                "BadText" => assert!(
                    text.is_empty() || !text.bytes().all(|b| BASE45.contains(&b)),
                    "{name}"
                ),
                other => panic!("{name}: unknown QR error {other}"),
            }
        }
    }
}
