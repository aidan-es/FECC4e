// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! An alias is the name art had before it was renamed. Allows compatibility with old save files.
use crate::asset::Asset;
use indexmap::IndexMap;
use std::sync::LazyLock;

type OldName = &'static str;
type CurrentName = &'static str;
static RENAMED_ARRAY: &[(OldName, CurrentName)] = &[
    ("AcherAlt2Iscaneus_Token", "ArcherAlt3_Token"),
    ("ArcherAlt2Iscaneus_Token", "ArcherAlt2_Token"),
    ("ArcherAlt3Iscaneus_Token", "ArcherAlt3_Token"),
    ("ArcherAltIscaneus_Token", "ArcherAlt_Token"),
    ("ArcherIscaneus_Token", "ArcherAlt4_Token"),
    ("AssassinAlt2Iscaneus_Token", "AssassinAlt2_Token"),
    ("AssassinAltIscaneus_Token", "AssassinAlt_Token"),
    ("AssassinIscaneus_Token", "AssassinAlt3_Token"),
    ("AstridBoxIscaneus_Token", "AstridBox_Token"),
    ("AstridIscaneus_Token", "Astrid_Token"),
    ("AugerersHood_Hair", "AugurersHood_Hair"),
    ("AugerersHood_HairBack", "AugurersHood_HairBack"),
    ("AugerersHoodSmall_Hair", "AugurersHoodSmall_Hair"),
    ("AugerersHoodSmall_HairBack", "AugurersHoodSmall_HairBack"),
    ("Axe2Iscaneus_Token", "AxeAlt_Token"),
    ("AxeAltIscaneus_Token", "AxeAlt_Token"),
    ("AxeFighterIscaneus_Token", "AxeFighter_Token"),
    ("AxeIscaneus_Token", "Axe_Token"),
    ("Bandit2_Armour", "Assassin_Armour"),
    ("Bandit2_Face", "Assassin_Face"),
    ("Bandit2_Hair", "Assassin_Hair"),
    ("Bandit2_HairBack", "Assassin_HairBack"),
    ("BerserkerIscaneus_Token", "BerserkerAlt2_Token"),
    ("BishopIscaneus_Token", "BishopAlt3_Token"),
    ("BowKnightAltIscaneus_Token", "BowKnightAlt_Token"),
    ("BowKnightIscaneus_Token", "BowKnight_Token"),
    ("BoydAltIscaneus_Token", "BoydAlt_Token"),
    ("BoydFighterIscaneus_Token", "BoydFighter_Token"),
    ("BoydIscaneus_Token", "Boyd_Token"),
    ("Brenden_Armour", "Brendan_Armour"),
    ("Brenden_Face", "Brendan_Face"),
    ("Brenden_Hair", "Brendan_Hair"),
    ("Brenden_HairBack", "Brendan_HairBack"),
    ("CavalierAxeAltIscaneus_Token", "CavalierAxeAlt_Token"),
    ("CavalierAxeIscaneus_Token", "CavalierAxe_Token"),
    ("CavalierBowIscaneus_Token", "CavalierBow_Token"),
    ("CavalierSwordAltIscaneus_Token", "CavalierSwordAlt_Token"),
    ("CavalierSwordIscaneus_Token", "CavalierSword_Token"),
    ("CavelierFemale_Token", "CavalierFemale_Token"),
    ("Celina_Armour", "Selena_Armour"),
    ("Celina_Face", "Selena_Face"),
    ("Celina_Hair", "Selena_Hair"),
    ("Celina_HairBack", "Selena_HairBack"),
    ("ChromAltIscaneus_Token", "ChromAlt_Token"),
    ("ChromIscaneus_Token", "Chrom_Token"),
    ("ClericIscaneus_Token", "ClericAlt2_Token"),
    ("DancerIscaneus_Token", "DancerAlt2_Token"),
    ("DarkKnightAltIscaneus_Token", "DarkKnightAlt_Token"),
    ("DarkKnightIscaneus_Token", "DarkKnight_Token"),
    ("DonnelIscaneus_Token", "Donnel_Token"),
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
    ("FalconKnightIscaneus_Token", "FalconKnight_Token"),
    ("FighterAlternateIscaneus_Token", "FighterAlt_Token"),
    ("FighterAltIscaneus_Token", "FighterAlt_Token"),
    ("FighterIscaneus_Token", "FighterAlt2_Token"),
    ("GeneralAxeIscaneus_Token", "GeneralAxe_Token"),
    ("GeneralHandaxeIscaneus_Token", "GeneralHandaxe_Token"),
    ("Gerrick_Armour", "Gerik_Armour"),
    ("Gerrick_Face", "Gerik_Face"),
    ("Gerrick_Hair", "Gerik_Hair"),
    ("Gerrick_HairBack", "Gerik_HairBack"),
    ("GreatKnightIscaneus_Token", "GreatKnightAlt2_Token"),
    ("GuySwordMaster_Token", "GuySwordmaster_Token"),
    ("HalberdierAltIscaneus_Token", "HalberdierAlt_Token"),
    ("HalberdierIscaneus_Token", "Halberdier_Token"),
    ("HectorGreatlord_Token", "HectorGreatLord_Token"),
    ("IkeGreatlordIscaneus_Token", "IkeGreatlord_Token"),
    ("IkeLordAltIscaneus_Token", "IkeLordAlt_Token"),
    ("IkeLordIscaneus_Token", "IkeLord_Token"),
    ("Inscaneus89_Token", "Iscaneus89_Token"),
    ("JourneymanAltIscaneus_Token", "JourneymanAlt_Token"),
    ("JourneymanIscaneus_Token", "JourneymanAlt2_Token"),
    ("Klien_Armour", "Klein_Armour"),
    ("Klien_Face", "Klein_Face"),
    ("Klien_Hair", "Klein_Hair"),
    ("Klien_HairBack", "Klein_HairBack"),
    ("KnightAxeIscaneus_Token", "KnightAxe_Token"),
    ("KnightIscaneus_Token", "KnightAlt2_Token"),
    ("KnightSpearIscaneus_Token", "KnightSpear_Token"),
    ("KnightSwordIscaneus_Token", "KnightSword_Token"),
    ("Legualt_Armour", "Legault_Armour"),
    ("Legualt_Face", "Legault_Face"),
    ("Legualt_Hair", "Legault_Hair"),
    ("Legualt_HairBack", "Legault_HairBack"),
    ("Loyd_Token", "Lloyd_Token"),
    ("LucinaIscaneus_Token", "Lucina_Token"),
    ("LynGreatlord_Token", "LynBladeLord_Token"),
    ("MageAltIscaneus_Token", "MageAlt_Token"),
    ("MageIscaneus_Token", "MageAlt2_Token"),
    ("MageKnight_Token", "MageKnightFemale_Token"),
    ("MercenaryIscaneus_Token", "MercenaryAlt2_Token"),
    ("MiaIscaneus_Token", "Mia_Token"),
    ("MiaMyrmidonIscaneus_Token", "MiaMyrmidon_Token"),
    ("MyrmidonAlt2Iscaneus_Token", "MyrmidonAlt2_Token"),
    ("MyrmidonAlt3Iscaneus_Token", "MyrmidonAlt3_Token"),
    ("MyrmidonAlternateIscaneus_Token", "MyrmidonAlt3_Token"),
    ("MyrmidonAltIscaneus_Token", "MyrmidonAlt_Token"),
    ("MyrmidonIscaneus_Token", "MyrmidonAlt4_Token"),
    (
        "NepheneeHalberdierIscaneus_Token",
        "NepheneeHalberdier_Token",
    ),
    ("NepheneeIscaneus_Token", "Nephenee_Token"),
    ("NinjaAltIscaneus_Token", "NinjaAlt_Token"),
    ("NinjaIscaneus_Token", "Ninja_Token"),
    ("OniAltIscaneus_Token", "OniAlt_Token"),
    ("OniIscaneus_Token", "Oni_Token"),
    ("PaladinIscaneus_Token", "PaladinAlt2_Token"),
    ("PaladinSpearIscaneus_Token", "PaladinSpear_Token"),
    ("PaladinSwordIscaneus_Token", "PaladinSword_Token"),
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
    ("PriestAltIscaneus_Token", "PriestAlt_Token"),
    ("PriestessAltIscaneus_Token", "PriestessAlt_Token"),
    ("PriestessIscaneus_Token", "Priestess_Token"),
    ("PriestIscaneus_Token", "PriestAlt2_Token"),
    ("PriestStaffIscaneus_Token", "PriestStaff_Token"),
    ("RobinAltIscaneus_Token", "RobinAlt_Token"),
    ("RobinIscaneus_Token", "Robin_Token"),
    ("RobinLevinIscaneus_Token", "RobinLevin_Token"),
    ("Roy_Young_Armour", "RoyYoung_Armour"),
    ("Roy_Young_Face", "RoyYoung_Face"),
    ("Roy_Young_Hair", "RoyYoung_Hair"),
    ("Roy_Young_HairBack", "RoyYoung_HairBack"),
    ("Rutoga_Armour", "Rutger_Armour"),
    ("Rutoga_Face", "Rutger_Face"),
    ("Rutoga_Hair", "Rutger_Hair"),
    ("Rutoga_HairBack", "Rutger_HairBack"),
    ("sage_casting_Token", "SageCasting_Token"),
    ("SageAltIscaneus_Token", "SageAlt_Token"),
    ("SageIscaneus_Token", "Sage_Token"),
    ("Sera_Armour", "Serra_Armour"),
    ("Sera_Face", "Serra_Face"),
    ("Sera_Hair", "Serra_Hair"),
    ("Sera_HairBack", "Serra_HairBack"),
    ("ShinonIscaneus_Token", "Shinon_Token"),
    ("SniperAlt2Iscaneus_Token", "SniperAlt2_Token"),
    ("SniperAlt3Iscaneus_Token", "SniperAlt3_Token"),
    ("SniperAltIscaneus_Token", "SniperAlt_Token"),
    ("SniperIscaneus_Token", "SniperAlt4_Token"),
    ("SolderAlt2Iscaneus_Token", "SoldierAlt2_Token"),
    ("SoldierAlt2Iscaneus_Token", "SoldierAlt2_Token"),
    ("SoldierAltIscaneus_Token", "SoldierAlt_Token"),
    ("SoldierIscaneus_Token", "SoldierAlt3_Token"),
    ("SorcererIscaneus_Token", "Sorcerer_Token"),
    ("SpearFighterAlt2Iscaneus_Token", "SpearFighterAlt2_Token"),
    ("SpearFighterAlt3Iscaneus_Token", "SpearFighterAlt3_Token"),
    ("SpearFighterAlt4Iscaneus_Token", "SpearFighterAlt4_Token"),
    ("SpearFighterAltIscaneus_Token", "SpearFighterAlt_Token"),
    ("SpearFighterIscaneus_Token", "SpearFighter_Token"),
    ("Sword2Iscaneus_Token", "SwordAlt4_Token"),
    ("SwordAlt2Iscaneus_Token", "SwordAlt2_Token"),
    ("SwordAlt3Iscaneus_Token", "SwordAlt3_Token"),
    ("SwordAlt4Iscaneus_Token", "SwordAlt4_Token"),
    ("SwordAlt5Iscaneus_Token", "SwordAlt5_Token"),
    ("SwordAltIscaneus_Token", "SwordAlt_Token"),
    ("SwordIscaneus_Token", "Sword_Token"),
    ("SwordMasterAlt2Iscaneus_Token", "SwordmasterAlt2_Token"),
    ("SwordmasterAlt2Iscaneus_Token", "SwordmasterAlt2_Token"),
    ("SwordMasterAltIscaneus_Token", "SwordmasterAlt_Token"),
    ("SwordmasterAltIscaneus_Token", "SwordmasterAlt_Token"),
    ("SwordMasterIscaneus_Token", "SwordmasterAlt3_Token"),
    ("SwordmasterIscaneus_Token", "SwordmasterAlt3_Token"),
    ("Teifling2_Hair", "Tiefling2_Hair"),
    ("Teifling2_HairBack", "Tiefling2_HairBack"),
    ("TharjaIscaneus_Token", "Tharja_Token"),
    ("ThiefIscaneus_Token", "ThiefAlt2_Token"),
    ("TormodIscaneus_Token", "Tormod_Token"),
    ("TricksterIscaneus_Token", "Trickster_Token"),
    ("TroubadourIscaneus_Token", "TroubadourAlt2_Token"),
    ("ValkyrieIscaneus_Token", "ValkyrieAlt2_Token"),
    ("Viking_Armour", "Viking2_Armour"),
    ("vikingHelmet3_Hair", "VikingHelmet3_Hair"),
    ("vikingHelmet3_HairBack", "VikingHelmet3_HairBack"),
    ("vikingHelmet4_Hair", "VikingHelmet4_Hair"),
    ("vikingHelmet4_HairBack", "VikingHelmet4_HairBack"),
    ("vikingHelmet5_Hair", "VikingHelmet5_Hair"),
    ("vikingHelmet5_HairBack", "VikingHelmet5_HairBack"),
    ("WarriorIscaneus_Token", "WarriorAlt2_Token"),
    ("ZiharkIscaneus_Token", "Zihark_Token"),
    ("ZiharkMyrmidonIscaneus_Token", "ZiharkMyrmidon_Token"),
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
        let library = face_library(&["Legault{IS}"]);

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
