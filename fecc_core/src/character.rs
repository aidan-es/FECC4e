// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::asset::{Asset, AssetType};
use crate::asset_aliases;
use crate::character::Colourable::{
    Accessory, Cloth, EyeAndBeard, Hair, Leather, Metal, Skin, Trim,
};
use crate::extensions::rgba::AdjustBrightness as _;
use crate::types::{Point, Rgba};
use indexmap::IndexMap;
use std::collections::HashMap;
use strum_macros::{Display, EnumIter};

/// Variants available for a Colourable
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
pub enum Shade {
    Light,
    Normal,
    Dark,
    Darker,
    Darkest,
    Base,
}

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

impl CharacterPart {
    fn normalise(&mut self, canvas_size: Point) {
        self.position.x /= canvas_size.x;
        self.position.y = 0.5 + (self.position.y - canvas_size.y / 2.0) / canvas_size.x;
        self.scale /= canvas_size.x;
    }

    fn denormalise(&mut self, canvas_size: Point) {
        self.position.x *= canvas_size.x;
        self.position.y = canvas_size.y / 2.0 + (self.position.y - 0.5) * canvas_size.x;
        self.scale *= canvas_size.x;
    }
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
                    CharacterPartColours::new(&Rgba::new(247, 173, 82, 255)),
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

    /// Converts the parts from canvas pixels to the units saved in files.
    ///
    /// Both axes use the canvas width as their unit, measured from the centre at (0.5, 0.5), so the
    /// layers stay lined up on a canvas of another shape. On a square canvas a position is the
    /// fraction of the canvas width and height.
    pub fn normalise(&mut self, portrait_size: Point, token_size: Point) {
        self.for_each_part_on_canvas(portrait_size, token_size, CharacterPart::normalise);
    }

    /// Converts the parts from the units saved in files to canvas pixels.
    pub fn denormalise(&mut self, portrait_size: Point, token_size: Point) {
        self.for_each_part_on_canvas(portrait_size, token_size, CharacterPart::denormalise);
    }

    /// Applies `f` to each part with the size of its canvas, skipping a canvas not laid out yet.
    fn for_each_part_on_canvas(
        &mut self,
        portrait_size: Point,
        token_size: Point,
        f: fn(&mut CharacterPart, Point),
    ) {
        for (canvas_size, part) in [
            (portrait_size, &mut self.armour),
            (portrait_size, &mut self.face),
            (portrait_size, &mut self.hair),
            (portrait_size, &mut self.hair_back),
            (portrait_size, &mut self.accessory),
            (token_size, &mut self.token),
        ] {
            if canvas_size.x > 0.0
                && canvas_size.y > 0.0
                && let Some(part) = part
            {
                f(part, canvas_size);
            }
        }
    }

    pub fn relink_assets(&mut self, asset_libraries: &HashMap<AssetType, IndexMap<String, Asset>>) {
        for (asset_type, part) in [
            (AssetType::Armour, &mut self.armour),
            (AssetType::Face, &mut self.face),
            (AssetType::Hair, &mut self.hair),
            (AssetType::HairBack, &mut self.hair_back),
            (AssetType::Accessory, &mut self.accessory),
            (AssetType::Token, &mut self.token),
        ] {
            if let Some(part) = part
                && let Some(asset) = asset_libraries
                    .get(&asset_type)
                    .and_then(|library| asset_aliases::resolve(library, &part.asset.id))
            {
                part.asset = asset.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Asset, AssetType};
    use crate::types::Rgba;

    fn part_at(x: f32, y: f32, scale: f32) -> CharacterPart {
        let asset = Asset::new(
            "Test".to_owned(),
            "art/Test_Face.png".into(),
            None,
            AssetType::Face,
        );
        CharacterPart {
            position: Point::new(x, y),
            scale,
            rotation: 0.0,
            flipped: false,
            asset,
        }
    }

    #[test]
    fn test_normalise_square_canvas_gives_fractions() {
        // Saves from square canvases are read the same as before non-square canvases existed.
        let mut character = Character {
            face: Some(part_at(30.0, 60.0, 3.0)),
            ..Default::default()
        };
        character.normalise(Point::new(96.0, 96.0), Point::new(64.0, 64.0));

        let face = character.face.unwrap();
        assert!((face.position.x - 30.0 / 96.0).abs() < 1e-6);
        assert!((face.position.y - 60.0 / 96.0).abs() < 1e-6);
        assert!((face.scale - 3.0 / 96.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalise_round_trips_on_non_square_canvas() {
        let canvas = Point::new(96.0, 80.0);
        let mut character = Character {
            face: Some(part_at(30.0, 60.0, 3.0)),
            ..Default::default()
        };
        character.normalise(canvas, Point::new(64.0, 64.0));
        character.denormalise(canvas, Point::new(64.0, 64.0));

        let face = character.face.unwrap();
        assert!((face.position.x - 30.0).abs() < 1e-4);
        assert!((face.position.y - 60.0).abs() < 1e-4);
        assert!((face.scale - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_denormalise_keeps_layers_lined_up_on_another_shape() {
        // Hair 24 art pixels left of and above the face, saved from a 96x96 canvas.
        let mut character = Character {
            face: Some(part_at(0.5, 0.5, 1.0 / 96.0)),
            hair: Some(part_at(0.25, 0.25, 1.0 / 96.0)),
            ..Default::default()
        };
        character.denormalise(Point::new(120.0, 100.0), Point::new(64.0, 64.0));

        let face = character.face.unwrap();
        let hair = character.hair.unwrap();
        let art_offset_x = (face.position.x - hair.position.x) / face.scale;
        let art_offset_y = (face.position.y - hair.position.y) / face.scale;
        assert!((art_offset_x - 24.0).abs() < 1e-4);
        assert!((art_offset_y - 24.0).abs() < 1e-4);
        assert!((face.position.y - 50.0).abs() < 1e-4, "stays centred");
    }

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
