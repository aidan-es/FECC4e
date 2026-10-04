// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! An alias is the name art had before it was renamed. Allows compatibility with old save files.
use crate::asset::Asset;
use indexmap::IndexMap;
use std::sync::LazyLock;

type OldName = &'static str;
type CurrentName = &'static str;
static RENAMED_ARRAY: &[(OldName, CurrentName)] = &[
    ("AcherAlt2Iscaneus_Token", "ArcherAlt3Iscaneus_Token"),
    ("AugerersHood_Hair", "AugurersHood_Hair"),
    ("AugerersHood_HairBack", "AugurersHood_HairBack"),
    ("AugerersHoodSmall_Hair", "AugurersHoodSmall_Hair"),
    ("AugerersHoodSmall_HairBack", "AugurersHoodSmall_HairBack"),
    ("Axe2Iscaneus_Token", "AxeAltIscaneus_Token"),
    ("Bandit2_Armour", "Assassin_Armour"),
    ("Bandit2_Face", "Assassin_Face"),
    ("Bandit2_Hair", "Assassin_Hair"),
    ("Bandit2_HairBack", "Assassin_HairBack"),
    ("Brenden_Armour", "Brendan_Armour"),
    ("Brenden_Face", "Brendan_Face"),
    ("Brenden_Hair", "Brendan_Hair"),
    ("Brenden_HairBack", "Brendan_HairBack"),
    ("CavelierFemale_Token", "CavalierFemale_Token"),
    ("Celina_Armour", "Selena_Armour"),
    ("Celina_Face", "Selena_Face"),
    ("Celina_Hair", "Selena_Hair"),
    ("Celina_HairBack", "Selena_HairBack"),
    ("Eirk_Armour", "ErikOld_Armour"),
    ("Eirk_Face", "ErikOld_Face"),
    ("EirkOld_Armour", "ErikOld_Armour"),
    ("EliwoodGreatlord_Token", "EliwoodKnightLord_Token"),
    ("Elphin_Token", "Elffin_Token"),
    ("EphraimGreatlord_Token", "EphraimGreatLord_Token"),
    ("Erika_Armour", "Eirika_Armour"),
    ("Erika_Face", "Eirika_Face"),
    ("Erika_Hair", "Eirika_Hair"),
    ("Erika_HairBack", "Eirika_HairBack"),
    ("Erika_Token", "Eirika_Token"),
    ("ErikaGreatlord_Token", "EirikaGreatLord_Token"),
    ("FighterAlternateIscaneus_Token", "FighterAltIscaneus_Token"),
    ("Gerrick_Armour", "Gerik_Armour"),
    ("Gerrick_Face", "Gerik_Face"),
    ("Gerrick_Hair", "Gerik_Hair"),
    ("Gerrick_HairBack", "Gerik_HairBack"),
    ("GuySwordMaster_Token", "GuySwordmaster_Token"),
    ("HectorGreatlord_Token", "HectorGreatLord_Token"),
    ("Inscaneus89_Token", "Iscaneus89_Token"),
    ("Klien_Armour", "Klein_Armour"),
    ("Klien_Face", "Klein_Face"),
    ("Klien_Hair", "Klein_Hair"),
    ("Klien_HairBack", "Klein_HairBack"),
    ("Legualt_Armour", "Legault_Armour"),
    ("Legualt_Face", "Legault_Face"),
    ("Legualt_Hair", "Legault_Hair"),
    ("Legualt_HairBack", "Legault_HairBack"),
    ("Loyd_Token", "Lloyd_Token"),
    ("LynGreatlord_Token", "LynBladeLord_Token"),
    ("MageKnight_Token", "MageKnightFemale_Token"),
    (
        "MyrmidonAlternateIscaneus_Token",
        "MyrmidonAlt3Iscaneus_Token",
    ),
    ("Peasant10_Armour", "Bandit5_Armour"),
    ("Peasant10_Face", "Bandit5_Face"),
    ("Peasant10_Hair", "Bandit5_Hair"),
    ("Peasant10_HairBack", "Bandit5_HairBack"),
    ("Peasant13_Armour", "Jan_Armour"),
    ("Peasant13_Face", "Jan_Face"),
    ("Peasant13_Hair", "Jan_Hair"),
    ("Peasant13_HairBack", "Jan_HairBack"),
    ("Peasant19_Armour", "Natalie_Armour"),
    ("Peasant19_Face", "Natalie_Face"),
    ("Peasant19_Hair", "Natalie_Hair"),
    ("Peasant19_HairBack", "Natalie_HairBack"),
    ("PeasantHood_Hair", "PeasantHood1_Hair"),
    ("PeasantHood_HairBack", "PeasantHood1_HairBack"),
    ("PegasusRider_Armour", "PegasusRider1_Armour"),
    ("PegasusRider_Face", "PegasusRider1_Face"),
    ("PegasusRider_Hair", "PegasusRider1_Hair"),
    ("PegasusRider_HairBack", "PegasusRider1_HairBack"),
    ("PegasusRiderToken_Token", "PegasusKnight_Token"),
    ("Roy_Young_Armour", "RoyYoung_Armour"),
    ("Roy_Young_Face", "RoyYoung_Face"),
    ("Roy_Young_Hair", "RoyYoung_Hair"),
    ("Roy_Young_HairBack", "RoyYoung_HairBack"),
    ("Rutoga_Armour", "Rutger_Armour"),
    ("Rutoga_Face", "Rutger_Face"),
    ("Rutoga_Hair", "Rutger_Hair"),
    ("Rutoga_HairBack", "Rutger_HairBack"),
    ("sage_casting_Token", "SageCasting_Token"),
    ("Sera_Armour", "Serra_Armour"),
    ("Sera_Face", "Serra_Face"),
    ("Sera_Hair", "Serra_Hair"),
    ("Sera_HairBack", "Serra_HairBack"),
    ("SolderAlt2Iscaneus_Token", "SoldierAlt2Iscaneus_Token"),
    ("Sword2Iscaneus_Token", "SwordAlt4Iscaneus_Token"),
    (
        "SwordMasterAlt2Iscaneus_Token",
        "SwordmasterAlt2Iscaneus_Token",
    ),
    (
        "SwordMasterAltIscaneus_Token",
        "SwordmasterAltIscaneus_Token",
    ),
    ("SwordMasterIscaneus_Token", "SwordmasterIscaneus_Token"),
    ("Teifling2_Hair", "Tiefling2_Hair"),
    ("Teifling2_HairBack", "Tiefling2_HairBack"),
    ("Viking_Armour", "Viking2_Armour"),
    ("vikingHelmet3_Hair", "VikingHelmet3_Hair"),
    ("vikingHelmet3_HairBack", "VikingHelmet3_HairBack"),
    ("vikingHelmet4_Hair", "VikingHelmet4_Hair"),
    ("vikingHelmet4_HairBack", "VikingHelmet4_HairBack"),
    ("vikingHelmet5_Hair", "VikingHelmet5_Hair"),
    ("vikingHelmet5_HairBack", "VikingHelmet5_HairBack"),
];

static RENAMED: LazyLock<IndexMap<OldName, CurrentName>> =
    LazyLock::new(|| RENAMED_ARRAY.iter().copied().collect());

/// Every retired asset name, mapped to the new name.
pub fn renamed() -> &'static IndexMap<OldName, CurrentName> {
    &RENAMED
}

/// Gets an asset by ID, following renames if the ID was retired.
pub fn resolve<'a>(library: &'a IndexMap<String, Asset>, id: &str) -> Option<&'a Asset> {
    library
        .get(id)
        .or_else(|| RENAMED.get(id).and_then(|new_id| library.get(*new_id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetType;
    use crate::file_io::repository_art_libraries;
    use std::path::PathBuf;

    fn face_library(names: &[&str]) -> IndexMap<String, Asset> {
        names
            .iter()
            .map(|name| {
                let path = PathBuf::from(format!("art/{name}_Face.png"));
                let asset = Asset::new((*name).to_owned(), path, None, AssetType::Face);
                (asset.id.clone(), asset)
            })
            .collect()
    }

    #[test]
    fn test_resolve_follows_renames() {
        let library = face_library(&["Legault"]);

        let asset = resolve(&library, "Legualt_Face");
        assert_eq!(asset.map(|asset| asset.id.as_str()), Some("Legault_Face"));
        assert_eq!(resolve(&library, "Missing_Face"), None);
    }

    #[test]
    fn test_resolve_prefers_saved_id() {
        // A file that still has a retired name keeps working.
        let library = face_library(&["Legault", "Legualt"]);

        let asset = resolve(&library, "Legualt_Face");
        assert_eq!(asset.map(|asset| asset.id.as_str()), Some("Legualt_Face"));
    }

    #[test]
    fn test_renamed_ids_point_at_current_art() {
        let libraries = repository_art_libraries();
        let in_art = |id: &str| libraries.values().any(|library| library.contains_key(id));

        for (old_id, new_id) in renamed() {
            assert!(!in_art(old_id), "{old_id} is retired but still in art/");
            assert!(
                in_art(new_id),
                "{old_id} is renamed to {new_id}, which is not in art/"
            );
            assert!(
                !renamed().contains_key(*new_id),
                "{old_id} is renamed to {new_id}, which is itself retired"
            );
            assert_eq!(
                Asset::parse_filename(old_id).map(|(_, asset_type)| asset_type),
                Asset::parse_filename(new_id).map(|(_, asset_type)| asset_type),
                "{old_id} is renamed to {new_id}, which is a different layer"
            );
        }
    }
}
