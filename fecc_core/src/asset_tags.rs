// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Reads the categories and contributors in art file names and filters assets by them.
//!
//! File names are in the format `Name[Category]{Contributor}_Type.png`, e.g. `Lyn[FE7]_Face.png`
//! or `ArcherAlt4{Iscaneus}_Token.png`. Tags can be repeated or hold a comma separated list, e.g.
//! `Bob[FE6][MyHack]{Jane, Sam}_Hair.png`.
//!
//! Tags are left out of an asset's id so that changing them doesn't break save files. This means
//! names must be unique without their tags.
use crate::asset::Asset;
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Represents the parts of an asset's name.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedName {
    /// Name without any tags, e.g. `Archer`.
    pub name: String,
    pub categories: Vec<String>,
    pub contributors: Vec<String>,
}

/// Parses a name such as `Lyn[FE7]{Someone}` into its name, categories and contributors.
pub fn parse_name(raw: &str) -> Result<ParsedName, String> {
    let mut name = String::new();
    let mut categories = Vec::new();
    let mut contributors = Vec::new();

    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        let (close, tags) = match c {
            '[' => (']', &mut categories),
            '{' => ('}', &mut contributors),
            ']' | '}' => return Err(format!("'{raw}' has an unopened '{c}'")),
            _ => {
                name.push(c);
                continue;
            }
        };
        let mut tag = String::new();
        loop {
            match chars.next() {
                Some(next) if next == close => break,
                Some('[' | ']' | '{' | '}') | None => {
                    return Err(format!("'{raw}' has an unclosed '{c}'"));
                }
                Some(next) => tag.push(next),
            }
        }
        for part in tag
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            if !tags.iter().any(|t: &String| same_tag(t, part)) {
                tags.push(part.to_owned());
            }
        }
    }

    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(format!("'{raw}' has no name outside its tags"));
    }

    Ok(ParsedName {
        name,
        categories,
        contributors,
    })
}

/// Fire Emblem game titles in release order, FE1 through FE18 for completeness.
const GAMES: &[&str] = &[
    "Shadow Dragon and the Blade of Light",
    "Gaiden",
    "Mystery of the Emblem",
    "Genealogy of the Holy War",
    "Thracia 776",
    "The Binding Blade",
    "The Blazing Blade",
    "The Sacred Stones",
    "Path of Radiance",
    "Radiant Dawn",
    "Shadow Dragon",
    "New Mystery of the Emblem",
    "Awakening",
    "Fates",
    "Shadows of Valentia",
    "Three Houses",
    "Engage",
    "Fortune's Weave",
];

/// Returns the game number for a category such as `FE7`.
fn game_number(category: &str) -> Option<usize> {
    let number = category
        .get(..2)
        .filter(|prefix| prefix.eq_ignore_ascii_case("FE"))
        .and_then(|_| category[2..].parse::<usize>().ok())?;
    (1..=GAMES.len()).contains(&number).then_some(number)
}

/// Returns the full title for a game category, e.g. `Fire Emblem: The Blazing Blade`.
pub fn category_title(category: &str) -> Option<String> {
    game_number(category).map(|number| format!("Fire Emblem: {}", GAMES[number - 1]))
}

/// Sorts games by number, followed by any other categories alphabetically.
pub fn compare_categories(a: &str, b: &str) -> Ordering {
    match (game_number(a), game_number(b)) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// Represents a choice in a filter row.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum TagChoice {
    Untagged,
    Tag(String),
}

impl TagChoice {
    /// Creates the choice for `tag`.
    pub fn tag(tag: &str) -> Self {
        Self::Tag(tag.to_lowercase())
    }
}

/// Filters the assets shown in the asset pane and used by the randomiser.
///
/// An asset matches a row if it has any of the row's choices. An empty row matches everything.
/// An asset must match both rows.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AssetFilter {
    pub categories: BTreeSet<TagChoice>,
    pub contributors: BTreeSet<TagChoice>,
}

impl AssetFilter {
    /// Checks whether an asset matches both rows.
    pub fn matches(&self, asset: &Asset) -> bool {
        row_matches(&self.categories, &asset.categories)
            && row_matches(&self.contributors, &asset.contributors)
    }
}

/// Checks whether an asset's tags match one row of the filter.
fn row_matches(chosen: &BTreeSet<TagChoice>, tags: &[String]) -> bool {
    chosen.is_empty()
        || (tags.is_empty() && chosen.contains(&TagChoice::Untagged))
        || tags.iter().any(|tag| chosen.contains(&TagChoice::tag(tag)))
}

/// Checks whether two tags are the same, ignoring case.
fn same_tag(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Lists the tags used by a set of assets, for the filter rows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvailableTags {
    pub categories: Vec<String>,
    pub any_uncategorised: bool,
    pub contributors: Vec<String>,
    pub any_without_contributor: bool,
}

impl AvailableTags {
    /// Collects the tags used by `assets`.
    pub fn from_assets<'a>(assets: impl IntoIterator<Item = &'a Asset>) -> Self {
        let mut available = Self::default();
        for asset in assets {
            available.any_uncategorised |= asset.categories.is_empty();
            available.any_without_contributor |= asset.contributors.is_empty();
            for (tags, seen) in [
                (&asset.categories, &mut available.categories),
                (&asset.contributors, &mut available.contributors),
            ] {
                for tag in tags {
                    if !seen.iter().any(|s| same_tag(s, tag)) {
                        seen.push(tag.clone());
                    }
                }
            }
        }
        available
            .categories
            .sort_by(|a, b| compare_categories(a, b));
        available
            .contributors
            .sort_by_key(|contributor| contributor.to_lowercase());
        available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parsed(raw: &str) -> ParsedName {
        parse_name(raw).unwrap()
    }

    fn asset(file_stem: &str) -> Asset {
        Asset::try_from(PathBuf::from(format!("art/{file_stem}.png")).as_path()).unwrap()
    }

    #[test]
    fn test_parse_name_without_tags() {
        let name = parsed("Lyn");
        assert_eq!(name.name, "Lyn");
        assert!(name.categories.is_empty());
        assert!(name.contributors.is_empty());
    }

    #[test]
    fn test_parse_name_with_tags() {
        let name = parsed("Sword[FE9]{Iscaneus}");
        assert_eq!(name.name, "Sword");
        assert_eq!(name.categories, ["FE9"]);
        assert_eq!(name.contributors, ["Iscaneus"]);
    }

    #[test]
    fn test_parse_name_lists_and_repeats() {
        let name = parsed("Bob[FE6][MyHack, fe6]{Jane,Sam}");
        assert_eq!(name.categories, ["FE6", "MyHack"]);
        assert_eq!(name.contributors, ["Jane", "Sam"]);
        assert_eq!(name.name, "Bob");
    }

    #[test]
    fn test_parse_name_tag_position_and_spaces_do_not_change_the_name() {
        for raw in [
            "Archer{Iscaneus}",
            "Archer {Iscaneus}",
            "{Iscaneus}Archer",
            "Archer[FE9]{ Iscaneus }",
        ] {
            assert_eq!(parsed(raw).name, "Archer", "for {raw}");
        }
    }

    #[test]
    fn test_parse_name_invalid() {
        for raw in [
            "Lyn[FE7",
            "Lyn]",
            "Lyn{A",
            "Lyn{A[B]}",
            "[FE7]",
            "{Iscaneus}",
        ] {
            assert!(parse_name(raw).is_err(), "{raw} should not parse");
        }
    }

    #[test]
    fn test_tags_are_not_part_of_the_id() {
        assert_eq!(asset("Lyn[FE7]_Face").id, "Lyn_Face");
        assert_eq!(asset("Archer{Iscaneus}_Token").id, "Archer_Token");
        assert_eq!(asset("Bob[FE6]{Jane}_Hair").id, "Bob_Hair");
    }

    #[test]
    fn test_hair_links_to_back_part_whatever_its_tags() {
        let hair = asset("Lyn[FE7]{IS}_Hair");
        assert_eq!(hair.back_part, Some(asset("Lyn_HairBack").id));
    }

    #[test]
    fn test_category_title_and_order() {
        assert_eq!(
            category_title("fe7").as_deref(),
            Some("Fire Emblem: The Blazing Blade")
        );
        assert_eq!(category_title("FE0"), None);
        assert_eq!(category_title("MyHack"), None);

        let mut categories = vec!["MyHack", "FE10", "Alpha", "FE6", "fe9"];
        categories.sort_by(|a, b| compare_categories(a, b));
        assert_eq!(categories, ["FE6", "fe9", "FE10", "Alpha", "MyHack"]);
    }

    #[test]
    fn test_filter_rows() {
        let lyn = asset("Lyn[FE7]_Face");
        let archer = asset("Archer{Iscaneus}_Token");
        let tiefling = asset("Tiefling1_Hair");

        let mut filter = AssetFilter::default();
        assert!([&lyn, &archer, &tiefling].iter().all(|a| filter.matches(a)));

        filter.categories.insert(TagChoice::tag("fe7"));
        assert!(filter.matches(&lyn));
        assert!(!filter.matches(&archer));

        filter.categories.insert(TagChoice::Untagged);
        assert!(filter.matches(&archer));
        assert!(filter.matches(&tiefling));

        filter.contributors.insert(TagChoice::tag("Iscaneus"));
        assert!(!filter.matches(&lyn));
        assert!(filter.matches(&archer));
        assert!(!filter.matches(&tiefling));
    }

    #[test]
    fn test_available_tags() {
        let assets = [
            asset("Lyn[FE7]_Face"),
            asset("Roy[fe6]{X}_Face"),
            asset("Bob[MyHack][FE6]_Face"),
        ];
        let available = AvailableTags::from_assets(&assets);
        assert_eq!(available.categories, ["fe6", "FE7", "MyHack"]);
        assert!(!available.any_uncategorised);
        assert_eq!(available.contributors, ["X"]);
        assert!(available.any_without_contributor);
    }
}
