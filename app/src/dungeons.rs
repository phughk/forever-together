//! Dungeon table (ADR 0006, spec section 5) and topic IDs (spec section 2).
//!
//! The table is `data/dungeons.json`, supplied by the author. It has no faction column and
//! nothing here looks at faction when recommending.

use crate::code::{Character, Faction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const TABLE_JSON: &str = include_str!("../data/dungeons.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dungeon {
    pub slug: String,
    pub name: String,
    pub level_min: u8,
    pub level_max: u8,
    /// `None`: no known minimum level, so only the level range limits the recommendation.
    pub min_level: Option<u8>,
    pub zone: String,
}

/// The 32-byte hashed search topic.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TopicId(pub [u8; 32]);

impl std::fmt::Debug for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TopicId({}..)",
            data_encoding::HEXLOWER.encode(&self.0[..4])
        )
    }
}

/// `SHA-256("wowft/1/" + region + "/" + realm + "/" + faction + "/" + slug + "/" + shard)`.
pub fn topic_id(region: u8, realm: &str, faction: Faction, slug: &str, shard: u32) -> TopicId {
    let s = format!("wowft/1/{region}/{realm}/{}/{slug}/{shard}", faction.slug());
    TopicId(Sha256::digest(s.as_bytes()).into())
}

pub fn load_table() -> Vec<Dungeon> {
    serde_json::from_str(TABLE_JSON).expect("data/dungeons.json is valid")
}

pub fn find<'a>(table: &'a [Dungeon], slug: &str) -> Option<&'a Dungeon> {
    table.iter().find(|d| d.slug == slug)
}

/// Dungeons recommended for a character. Keeps entries with `minLevel <= L` (a `null` minimum always passes) and
/// `levelMin - 3 <= L <= levelMax + 3`. Sorted by: zone equal to the character's zone (ASCII
/// lowercased) first, then distance of L from the middle of the range, then slug.
pub fn recommend<'a>(character: &Character, table: &'a [Dungeon]) -> Vec<&'a Dungeon> {
    let level = character.level as i32;
    let zone = character.zone.to_ascii_lowercase();
    let mut list: Vec<&Dungeon> = table
        .iter()
        .filter(|d| {
            d.min_level.is_none_or(|m| m as i32 <= level)
                && d.level_min as i32 - 3 <= level
                && level <= d.level_max as i32 + 3
        })
        .collect();
    list.sort_by_key(|d| {
        let same_zone = !zone.is_empty() && d.zone.to_ascii_lowercase() == zone;
        // Twice the distance from the middle, to stay in integers.
        let dist = (2 * level - (d.level_min as i32 + d.level_max as i32)).abs();
        (!same_zone, dist, d.slug.clone())
    });
    list
}

/// Topics (shard 0) for the character and the chosen dungeons.
pub fn topics_for(character: &Character, dungeons: &[&Dungeon]) -> Vec<TopicId> {
    dungeons
        .iter()
        .map(|d| {
            topic_id(
                character.region,
                &character.realm,
                character.faction,
                &d.slug,
                0,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::test_encoder::sample_character;

    fn who(level: u8, zone: &str) -> Character {
        Character {
            level,
            zone: zone.into(),
            ..sample_character()
        }
    }

    fn slugs(c: &Character) -> Vec<String> {
        recommend(c, &load_table())
            .iter()
            .map(|d| d.slug.clone())
            .collect()
    }

    #[test]
    fn table_has_29_entries() {
        let t = load_table();
        assert_eq!(t.len(), 29);
        // Nine entries have no known minimum level.
        assert_eq!(t.iter().filter(|d| d.min_level.is_none()).count(), 9);
        for d in &t {
            assert!(d.level_min <= d.level_max);
        }
        let mut seen: Vec<_> = t.iter().map(|d| d.slug.as_str()).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 29, "slugs are unique");
    }

    #[test]
    fn slugs_are_the_slugified_names() {
        for d in load_table() {
            let want: String = d
                .name
                .to_ascii_lowercase()
                .replace('\'', "")
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            assert_eq!(d.slug, want);
        }
    }

    // Expected lists below were worked out by hand from data/dungeons.json:
    // window [levelMin-3, levelMax+3] and minLevel <= L; zone match first; then
    // |2L - (levelMin+levelMax)| ascending; then slug.

    #[test]
    fn barrens_level_15() {
        // wailing 12..28 (dist 10, zone); hall of thanes 10..21 (dist 1, no minimum), ragefire
        // 10..21 (dist 1, slug order after hall); ruins of lordaeron 12..23 (5); deadmines 15..26 (11).
        assert_eq!(
            slugs(&who(15, "The Barrens")),
            [
                "wailing-caverns",
                "hall-of-thanes",
                "ragefire-chasm",
                "ruins-of-lordaeron",
                "the-deadmines"
            ]
        );
    }

    #[test]
    fn barrens_level_25() {
        assert_eq!(
            slugs(&who(25, "The Barrens")),
            [
                "wailing-caverns", // zone
                "shadowfang-keep", // dist 2, ties with the stockade, slug order
                "the-stockade",
                "excavation-site",   // 3
                "blackfathom-deeps", // 6
                "the-deadmines",     // 9
                "city-of-dalaran",   // 11
                "scarlet-monastery", // 23
            ]
        );
    }

    #[test]
    fn barrens_level_38() {
        assert_eq!(
            slugs(&who(38, "the barrens")), // zone compare ignores case
            [
                "razorfen-kraul",    // zone, dist 6
                "razorfen-downs",    // zone, dist 14
                "the-drowned-city",  // 1
                "scarlet-monastery", // 3
                "gnomeregan",        // 9, ties with krol'dok, slug order
                "kroldok-stronghold",
            ]
        );
    }

    #[test]
    fn end_game_character_in_eastern_plaguelands() {
        assert_eq!(
            slugs(&who(60, "Eastern Plaguelands")),
            [
                "stratholme", // zone
                "dire-maul",  // dist 2, slug order among five
                "scholomance",
                "shapers-terrace",
                "upper-blackrock-spire",
                "blackmaw-hold",         // 5
                "lower-blackrock-spire", // 5
                "blackrock-depths",      // 8
                "temple-of-atalhakkar",  // 10
                "maraudon",              // 18
            ]
        );
    }

    #[test]
    fn a_missing_minimum_level_means_no_minimum() {
        // Hall of Thanes has no known minimum: level 10 is the bottom of its range window
        // (13 - 3), so it appears alongside ragefire-chasm (minimum 10).
        assert_eq!(
            slugs(&who(10, "Orgrimmar")),
            ["ragefire-chasm", "hall-of-thanes"]
        );
        let t = load_table();
        assert!(find(&t, "hall-of-thanes").unwrap().min_level.is_none());
    }

    #[test]
    fn below_every_level_range_nothing_is_recommended() {
        assert!(slugs(&who(9, "Orgrimmar")).is_empty());
        assert!(slugs(&who(5, "Durotar")).is_empty());
    }

    #[test]
    fn faction_does_not_change_the_recommendation() {
        let a = who(25, "The Barrens");
        let h = Character {
            faction: Faction::Horde,
            ..a.clone()
        };
        assert_eq!(slugs(&a), slugs(&h));
    }

    #[test]
    fn topics_differ_by_every_part_and_are_stable() {
        let t = load_table();
        let c = sample_character();
        let dm = find(&t, "the-deadmines").unwrap();
        let wc = find(&t, "wailing-caverns").unwrap();
        let topics = topics_for(&c, &[dm, wc]);
        assert_eq!(topics.len(), 2);
        assert_ne!(topics[0], topics[1]);
        assert_eq!(
            topics[0],
            topic_id(3, "5", Faction::Alliance, "the-deadmines", 0)
        );
        assert_ne!(
            topics[0],
            topic_id(3, "5", Faction::Horde, "the-deadmines", 0)
        );
        assert_ne!(
            topics[0],
            topic_id(2, "5", Faction::Alliance, "the-deadmines", 0)
        );
        assert_ne!(
            topics[0],
            topic_id(3, "6", Faction::Alliance, "the-deadmines", 0)
        );
        assert_ne!(
            topics[0],
            topic_id(3, "5", Faction::Alliance, "the-deadmines", 1)
        );
    }
}
