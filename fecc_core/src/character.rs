// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::asset::{Asset, AssetType};
use crate::character::Colourable::{
    Accessory, Cloth, EyeAndBeard, Hair, Leather, Metal, Skin, Trim,
};
use crate::extensions::rgba::AdjustBrightness as _;
use crate::types::{Point, Rgba};
use std::collections::HashMap;
use strum_macros::{Display, EnumIter};

/// Represents a distinct, colourable area of a character asset.
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
    serde::Deserialize,
    serde::Serialize,
)]
pub enum Colourable {
    Hair,
    #[strum(to_string = "Eye & Beard")]
    EyeAndBeard,
    Skin,
    Metal,
    Trim,
    Cloth,
    Leather,
    Accessory,
    Outline,
}

/// A cyclical iterator over a collection of colours.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ColourPalette {
    colours: Vec<Rgba>,
    current_index: usize,
}

impl ColourPalette {
    pub fn new(colours: Vec<Rgba>) -> Self {
        Self {
            colours,
            current_index: 0,
        }
    }

    pub fn current(&self) -> &Rgba {
        self.colours.get(self.current_index).unwrap_or_else(|| {
            log::error!("ColourPalette::current: Colour index out of bounds!");
            const DEBUG_COLOR: Rgba = Rgba::new(255, 0, 255, 255);
            &DEBUG_COLOR
        })
    }

    pub fn next_cyclic(&mut self) -> &Rgba {
        if self.colours.is_empty() {
            const DEBUG_COLOR: Rgba = Rgba::new(255, 0, 255, 255);
            return &DEBUG_COLOR;
        }
        self.current_index = (self.current_index + 1) % self.colours.len();
        self.current()
    }

    pub fn peek(&self) -> &Rgba {
        if self.colours.is_empty() {
            const DEBUG_COLOR: Rgba = Rgba::new(255, 0, 255, 255);
            return &DEBUG_COLOR;
        }
        let peak_index = (self.current_index + 1) % self.colours.len();
        &self.colours[peak_index]
    }

    pub fn colours(&self) -> &Vec<Rgba> {
        &self.colours
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct CharacterPart {
    pub position: Point,
    pub scale: f32,
    pub rotation: f32,
    #[serde(default)]
    pub flipped: bool,
    pub asset: Asset,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CharacterPartColours {
    pub lighter: Rgba,
    pub neutral: Rgba,
    pub darker: Rgba,
    pub darker_darker: Rgba,
    pub darker_darker_darker: Rgba,
    pub base: Rgba,
}

impl CharacterPartColours {
    pub fn new(colour: &Rgba) -> Self {
        let mut character_part_colours = Self {
            base: *colour,
            ..Default::default()
        };
        character_part_colours.derive_all_colours();
        character_part_colours
    }

    pub fn derive_all_colours(&mut self) {
        self.lighter = self.base.brighter();
        self.neutral = self.base;
        self.darker = self.base.darker();
        self.darker_darker = self.base.darker().darker();
        self.darker_darker_darker = self.base.darker().darker().darker();
    }

    pub fn set(&mut self, colour: Rgba) {
        self.base = colour;
        self.derive_all_colours();
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub struct Outlines {
    outline_colours: HashMap<AssetType, Rgba>,
}

impl Outlines {
    pub fn new() -> Self {
        Self {
            outline_colours: [
                (AssetType::Armour, Rgba::new(56, 32, 64, 255)),
                (AssetType::Face, Rgba::new(56, 32, 64, 255)),
                (AssetType::Hair, Rgba::new(56, 32, 64, 255)),
                (AssetType::Accessory, Rgba::new(56, 32, 64, 255)),
                (AssetType::Token, Rgba::new(56, 32, 64, 255)),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn set_outline_colour(&mut self, asset_type: AssetType, colour: &Rgba) {
        if asset_type == AssetType::HairBack {
            self.outline_colours.insert(AssetType::Hair, *colour);
        } else {
            self.outline_colours.insert(asset_type, *colour);
        }
    }

    pub fn get_outline_colour(&self, asset_type: AssetType) -> Rgba {
        if asset_type == AssetType::HairBack {
            *self
                .outline_colours
                .get(&AssetType::Hair)
                .unwrap_or(&Rgba::BLACK)
        } else {
            *self
                .outline_colours
                .get(&asset_type)
                .unwrap_or(&Rgba::BLACK)
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(default)]
pub struct Character {
    pub name: String,
    pub armour: Option<CharacterPart>,
    pub face: Option<CharacterPart>,
    pub hair: Option<CharacterPart>,
    pub hair_back: Option<CharacterPart>,
    pub accessory: Option<CharacterPart>,
    pub token: Option<CharacterPart>,
    pub character_colours: HashMap<Colourable, CharacterPartColours>,
    pub outline_colours: Outlines,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: String::new(),
            armour: None,
            face: None,
            hair: None,
            hair_back: None,
            accessory: None,
            token: None,
            character_colours: [
                (
                    Hair,
                    CharacterPartColours::new(&Rgba::new(224, 216, 64, 255)),
                ),
                (
                    EyeAndBeard,
                    CharacterPartColours::new(&Rgba::new(64, 50, 25, 255)),
                ),
                (
                    Skin,
                    CharacterPartColours::new(&Rgba::new(248, 248, 192, 255)),
                ),
                (
                    Metal,
                    CharacterPartColours::new(&Rgba::new(100, 100, 100, 255)),
                ),
                (
                    Trim,
                    CharacterPartColours::new(&Rgba::new(247, 173, 82, 255)),
                ),
                (
                    Cloth,
                    CharacterPartColours::new(&Rgba::new(82, 82, 115, 255)),
                ),
                (
                    Leather,
                    CharacterPartColours::new(&Rgba::new(148, 100, 66, 255)),
                ),
                (
                    Accessory,
                    CharacterPartColours::new(&Rgba::new(0, 0, 0, 255)),
                ),
            ]
            .into_iter()
            .collect(),
            outline_colours: Outlines::new(),
        }
    }
}

impl Character {
    pub fn get_character_part(&self, asset_type: &AssetType) -> Option<CharacterPart> {
        match asset_type {
            AssetType::Armour => self.armour.clone(),
            AssetType::Face => self.face.clone(),
            AssetType::Hair => self.hair.clone(),
            AssetType::HairBack => self.hair_back.clone(),
            AssetType::Accessory => self.accessory.clone(),
            AssetType::Token => self.token.clone(),
        }
    }

    pub fn set_character_part(&mut self, asset_type: &AssetType, character_part: CharacterPart) {
        match asset_type {
            AssetType::Armour => self.armour = Some(character_part),
            AssetType::Face => self.face = Some(character_part),
            AssetType::Hair => self.hair = Some(character_part),
            AssetType::HairBack => self.hair_back = Some(character_part),
            AssetType::Accessory => self.accessory = Some(character_part),
            AssetType::Token => self.token = Some(character_part),
        }
    }

    pub fn remove_character_part(&mut self, asset_type: &AssetType) {
        match asset_type {
            AssetType::Armour => self.armour = None,
            AssetType::Face => self.face = None,
            AssetType::Hair => self.hair = None,
            AssetType::HairBack => self.hair_back = None,
            AssetType::Accessory => self.accessory = None,
            AssetType::Token => self.token = None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Asset, AssetType};
    use crate::types::Rgba;

    #[test]
    fn test_colour_palette_cyclic() {
        let colours = vec![
            Rgba::new(255, 0, 0, 255),
            Rgba::new(0, 255, 0, 255),
            Rgba::new(0, 0, 255, 255),
        ];
        let mut palette = ColourPalette::new(colours.clone());

        assert_eq!(*palette.current(), colours[0]);
        assert_eq!(*palette.peek(), colours[1]);

        assert_eq!(*palette.next_cyclic(), colours[1]);
        assert_eq!(*palette.current(), colours[1]);
        assert_eq!(*palette.peek(), colours[2]);

        assert_eq!(*palette.next_cyclic(), colours[2]);
        assert_eq!(*palette.current(), colours[2]);
        assert_eq!(*palette.peek(), colours[0]);

        assert_eq!(*palette.next_cyclic(), colours[0]);
    }

    #[test]
    fn test_colour_palette_empty() {
        let mut palette = ColourPalette::new(vec![]);
        let debug_color = Rgba::new(255, 0, 255, 255);

        assert_eq!(*palette.current(), debug_color);
        assert_eq!(*palette.peek(), debug_color);
        assert_eq!(*palette.next_cyclic(), debug_color);
    }

    #[test]
    fn test_character_part_colours_derive() {
        let base = Rgba::new(100, 100, 100, 255);
        let colours = CharacterPartColours::new(&base);

        assert_eq!(colours.base, base);
        assert_eq!(colours.neutral, base);
        // Check that derived colours are different (assuming implementation works)
        assert_ne!(colours.lighter, base);
        assert_ne!(colours.darker, base);
    }

    #[test]
    fn test_character_part_colours_set() {
        let initial_base = Rgba::new(100, 100, 100, 255);
        let mut colours = CharacterPartColours::new(&initial_base);

        let new_base = Rgba::new(200, 200, 200, 255);
        colours.set(new_base);

        assert_eq!(colours.base, new_base);
        assert_ne!(colours.base, initial_base);
        // Verify derived colors updated
        assert_ne!(colours.lighter, initial_base.brighter());
        assert_eq!(colours.lighter, new_base.brighter());
    }

    #[test]
    fn test_outlines_hair_logic() {
        let mut outlines = Outlines::new();
        let color = Rgba::new(10, 20, 30, 255);

        outlines.set_outline_colour(AssetType::HairBack, &color);

        // Setting HairBack should set Hair
        assert_eq!(outlines.get_outline_colour(AssetType::Hair), color);

        // Getting HairBack should return the Hair's colour
        assert_eq!(outlines.get_outline_colour(AssetType::HairBack), color);
    }

    #[test]
    fn test_outlines_default() {
        let outlines = Outlines::new();
        // Check the default outline colour for Face (defined in new())
        let default_color = Rgba::new(56, 32, 64, 255);
        assert_eq!(outlines.get_outline_colour(AssetType::Face), default_color);
    }

    #[test]
    fn test_character_part_management() {
        let mut character = Character::default();
        let part = CharacterPart {
            position: Point::new(0.0, 0.0),
            scale: 1.0,
            rotation: 0.0,
            flipped: false,
            asset: Asset::new(
                "test".to_string(),
                std::path::PathBuf::new(),
                None,
                AssetType::Face,
            ),
        };

        character.set_character_part(&AssetType::Face, part.clone());
        assert!(character.face.is_some());
        assert!(character.get_character_part(&AssetType::Face).is_some());

        character.remove_character_part(&AssetType::Face);
        assert!(character.face.is_none());
        assert!(character.get_character_part(&AssetType::Face).is_none());
    }
}
