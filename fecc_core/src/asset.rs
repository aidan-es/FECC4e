// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use image::RgbaImage;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use strum::IntoEnumIterator as _;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

/// Represents the layers of a character sprite.
///
/// The order of variants determines the default drawing order, from bottom to top.
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    EnumIter,
    Display,
    Ord,
    PartialOrd,
    IntoStaticStr,
    Default,
    serde::Deserialize,
    serde::Serialize,
    EnumString,
)]
pub enum AssetType {
    HairBack,
    Armour,
    Face,
    Hair,
    Accessory,
    #[default]
    Token,
}

impl AssetType {
    /// Returns an iterator over asset types that are selectable in the UI.
    ///
    /// `HairBack` is excluded.
    pub fn get_selectable_part_types() -> impl Iterator<Item = Self> {
        Self::iter().filter(|&p| p != Self::HairBack)
    }
}

/// Represents a single loadable asset.
///
/// Contains metadata about an asset, including its name, type, and file path.
/// For hair assets, it may include a reference to a corresponding back part.
/// Image data is loaded on demand.
#[derive(Clone, serde::Deserialize, serde::Serialize, Eq, PartialEq, Default, Debug)]
pub struct Asset {
    /// Form is `name_type`, e.g. `MyAsset_Face`.
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub back_part: Option<String>,
    pub asset_type: AssetType,
    #[serde(skip)]
    pub image_data: Option<Arc<RgbaImage>>,
}

impl Asset {
    pub fn new(
        name: String,
        path: PathBuf,
        back_part: Option<String>,
        asset_type: AssetType,
    ) -> Self {
        Self {
            id: name.clone() + "_" + &*asset_type.to_string(),
            name,
            path,
            back_part,
            asset_type,
            image_data: None,
        }
    }

    /// Parses a filename to extract the asset's name and type.
    ///
    /// Filenames are expected to be in the format `Name_Type`.
    pub fn parse_filename(filename: &str) -> Result<(&str, AssetType), String> {
        let (name, asset_type_str) = filename
            .rsplit_once('_')
            .ok_or_else(|| format!("Filename '{filename}' does not contain '_' separator"))?;

        let asset_type = match asset_type_str {
            "Armour" => AssetType::Armour,
            "Face" => AssetType::Face,
            "Hair" => AssetType::Hair,
            "Accessory" => AssetType::Accessory,
            "Token" => AssetType::Token,
            "HairBack" => AssetType::HairBack,
            _ => {
                return Err(format!("Unknown asset type in filename: {asset_type_str}"));
            }
        };

        Ok((name, asset_type))
    }

    /// Creates an `Asset` from a filename and image bytes.
    pub fn try_from_bytes(filename: &str, bytes: &[u8]) -> Result<Self, String> {
        let (name, asset_type) = Self::parse_filename(filename.trim_end_matches(".png"))?;

        let back_part_id = if asset_type == AssetType::Hair {
            Some(format!("{}Back.png", filename.trim_end_matches(".png")))
        } else {
            None
        };

        // Create a virtual path for the user asset
        let path = PathBuf::from(format!("user-asset://{filename}"));

        let mut asset = Self::new(name.to_owned(), path, back_part_id, asset_type);

        let image = image::load_from_memory(bytes)
            .map_err(|e| e.to_string())?
            .to_rgba8();

        asset.image_data = Some(Arc::new(image));

        Ok(asset)
    }
}

impl TryFrom<&Path> for Asset {
    type Error = String;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "Invalid filename".to_owned())?;

        let (name, asset_type) = Self::parse_filename(filename)?;

        let back_part_id = if asset_type == AssetType::Hair {
            let hair_back_id = filename.to_owned() + "Back";
            Some(hair_back_id)
        } else {
            None
        };

        Ok(Self::new(
            name.to_owned(),
            path.to_path_buf(),
            back_part_id,
            asset_type,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_filename_valid() {
        assert_eq!(
            Asset::parse_filename("MyAsset_Face"),
            Ok(("MyAsset", AssetType::Face))
        );
        assert_eq!(
            Asset::parse_filename("Cool_Hair"),
            Ok(("Cool", AssetType::Hair))
        );
        assert_eq!(
            Asset::parse_filename("Armor_Armour"),
            Ok(("Armor", AssetType::Armour))
        );
    }

    #[test]
    fn test_parse_filename_invalid() {
        assert!(Asset::parse_filename("InvalidFilename").is_err());
        assert!(Asset::parse_filename("Name_UnknownType").is_err());
    }

    #[test]
    fn test_asset_new() {
        let path = PathBuf::from("path/to/asset.png");
        let asset = Asset::new(
            "Test".to_owned(),
            path.clone(),
            Some("Back".to_owned()),
            AssetType::Face,
        );

        assert_eq!(asset.name, "Test");
        assert_eq!(asset.path, path);
        assert_eq!(asset.back_part, Some("Back".to_owned()));
        assert_eq!(asset.asset_type, AssetType::Face);
        assert_eq!(asset.id, "Test_Face");
    }

    #[test]
    fn test_try_from_path() {
        let path = PathBuf::from("assets/Test_Face.png");
        let asset = Asset::try_from(path.as_path()).expect("Failed to create asset from path");

        assert_eq!(asset.name, "Test");
        assert_eq!(asset.asset_type, AssetType::Face);
        assert_eq!(asset.back_part, None);
    }

    #[test]
    fn test_try_from_path_hair_back() {
        let path = PathBuf::from("assets/Style_Hair.png");
        let asset = Asset::try_from(path.as_path()).expect("Failed to create asset from path");

        assert_eq!(asset.name, "Style");
        assert_eq!(asset.asset_type, AssetType::Hair);
        assert_eq!(asset.back_part, Some("Style_HairBack".to_owned()));
    }

    #[test]
    fn test_asset_type_selectable() {
        let selectable: Vec<AssetType> = AssetType::get_selectable_part_types().collect();
        assert!(selectable.contains(&AssetType::Face));
        assert!(selectable.contains(&AssetType::Hair));
        assert!(!selectable.contains(&AssetType::HairBack));
    }

    #[test]
    fn test_try_from_bytes_success() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        let asset =
            Asset::try_from_bytes("Test_Face.png", &bytes).expect("Failed to load from bytes");
        assert_eq!(asset.name, "Test");
        assert_eq!(asset.asset_type, AssetType::Face);
        assert!(asset.image_data.is_some());
    }

    #[test]
    fn test_try_from_bytes_hair_back_logic() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        let asset =
            Asset::try_from_bytes("Style_Hair.png", &bytes).expect("Failed to load from bytes");
        assert_eq!(asset.name, "Style");
        assert_eq!(asset.asset_type, AssetType::Hair);
        assert_eq!(asset.back_part, Some("Style_HairBack.png".to_string()));
    }

    #[test]
    fn test_try_from_bytes_invalid_image() {
        let bytes = vec![0, 1, 2, 3];
        let result = Asset::try_from_bytes("Test_Face.png", &bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_bytes_invalid_filename() {
        let bytes = vec![];
        let result = Asset::try_from_bytes("InvalidName.png", &bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_path_invalid_filename_structure() {
        let path = PathBuf::from("assets/InvalidName.png");
        let result = Asset::try_from(path.as_path());
        assert!(result.is_err());
    }
}
