// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::asset::AssetType;
use crate::character::Character;
use crate::recolour::recolour;
use crate::types::Point;
use image::{Rgba, RgbaImage, imageops};
use imageproc::geometric_transformations::{Interpolation, rotate_about_center};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Defines the output dimensions for the exported character images.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Display)]
pub enum ExportSize {
    Half,
    Original,
    Double,
    ROMHack,
    Custom {
        portrait: (u32, u32),
        token: (u32, u32),
    },
}

impl ExportSize {
    /// Returns a formatted string for display in the UI.
    ///
    /// The string includes the export size name, portrait dimensions, and token dimensions.
    pub fn display_name(&self) -> String {
        match self {
            Self::Custom { portrait, token } => {
                format!(
                    "Custom ({}x{}) ({}x{})",
                    portrait.0, portrait.1, token.0, token.1
                )
            }
            _ => format!(
                "{} ({}x{}) ({}x{})",
                self,
                self.portrait().0,
                self.portrait().1,
                self.token().0,
                self.token().1
            ),
        }
    }

    /// Returns the portrait dimensions for the selected export size.
    pub fn portrait(&self) -> (u32, u32) {
        match self {
            Self::Half => (48, 48),
            Self::Original => (96, 96),
            Self::Double => (192, 192),
            Self::ROMHack => (96, 80),
            Self::Custom { portrait, .. } => *portrait,
        }
    }

    /// Returns the token dimensions for the selected export size.
    pub fn token(&self) -> (u32, u32) {
        match self {
            Self::Half => (32, 32),
            Self::Original => (64, 64),
            Self::Double => (128, 128),
            Self::ROMHack => (64, 64),
            Self::Custom { token, .. } => *token,
        }
    }
}

/// Exports a character portrait or token as an `RgbaImage`.
///
/// Composites the character's parts into a single image, applying
/// the necessary transformations to match their appearance on the UI canvas.
pub fn export_character(
    character: &Character,
    parts_to_draw: &[AssetType],
    output_size: (u32, u32),
    ui_canvas_size: Point,
) -> Option<RgbaImage> {
    if ui_canvas_size.x <= 0.0
        || ui_canvas_size.y <= 0.0
        || output_size.0 == 0
        || output_size.1 == 0
    {
        return None;
    }

    // Uniform scaling factor from UI canvas to exported image.
    let export_scale =
        (output_size.0 as f32 / ui_canvas_size.x).max(output_size.1 as f32 / ui_canvas_size.y);

    // Create an oversized buffer to prevent clipping during rotation and scaling.
    let buffer_dimension = (output_size.0.max(output_size.1) * 2).max(1);
    let mut buffer = RgbaImage::new(buffer_dimension, buffer_dimension);
    let buffer_centre_x = buffer_dimension / 2;
    let buffer_centre_y = buffer_dimension / 2;

    for part_type in parts_to_draw {
        if let Some(part) = character.get_character_part(part_type)
            && let Some(original_image_data) = part.asset.image_data.as_ref()
        {
            let mut part_image = (**original_image_data).clone();

            recolour(
                &mut part_image,
                *part_type,
                &character.character_colours,
                &character.outline_colours,
            );

            // Scale the asset image based on its UI scale and the export scale.
            let final_scale_factor = part.scale * export_scale;
            let scaled_width = (part_image.width() as f32 * final_scale_factor).round() as u32;
            let scaled_height = (part_image.height() as f32 * final_scale_factor).round() as u32;

            if scaled_width == 0 || scaled_height == 0 {
                continue;
            }

            let mut scaled_image = imageops::resize(
                &part_image,
                scaled_width,
                scaled_height,
                imageops::FilterType::Nearest,
            );

            if part.flipped {
                scaled_image = imageops::flip_horizontal(&scaled_image);
            }

            let rotated_image = rotate_about_center(
                &scaled_image,
                part.rotation,
                Interpolation::Nearest,
                imageproc::geometric_transformations::Border::Constant(Rgba([0, 0, 0, 0])),
            );

            // Calculate centre position relative to canvas centre
            let offset_from_center_x = (part.position.x - (ui_canvas_size.x / 2.0)) * export_scale;
            let offset_from_center_y = (part.position.y - (ui_canvas_size.y / 2.0)) * export_scale;

            let top_left_x = (buffer_centre_x as f32 + offset_from_center_x)
                - (rotated_image.width() as f32 / 2.0);
            let top_left_y = (buffer_centre_y as f32 + offset_from_center_y)
                - (rotated_image.height() as f32 / 2.0);

            imageops::overlay(
                &mut buffer,
                &rotated_image,
                top_left_x.round() as i64,
                top_left_y.round() as i64,
            );
        }
    }

    let crop_x = buffer_centre_x - (output_size.0 / 2);
    let crop_y = buffer_centre_y - (output_size.1 / 2);
    Some(imageops::crop_imm(&buffer, crop_x, crop_y, output_size.0, output_size.1).to_image())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Asset, AssetType};
    use crate::character::{Character, CharacterPart};
    use image::Rgba;
    use std::sync::Arc;

    #[test]
    fn test_export_size_display_name() {
        assert_eq!(ExportSize::Half.display_name(), "Half (48x48) (32x32)");
        assert_eq!(
            ExportSize::Original.display_name(),
            "Original (96x96) (64x64)"
        );
        assert_eq!(
            ExportSize::Double.display_name(),
            "Double (192x192) (128x128)"
        );
    }

    #[test]
    fn test_export_size_dimensions() {
        assert_eq!(ExportSize::Half.portrait(), (48, 48));
        assert_eq!(ExportSize::Half.token(), (32, 32));

        assert_eq!(ExportSize::Original.portrait(), (96, 96));
        assert_eq!(ExportSize::Double.portrait(), (192, 192));
    }

    #[test]
    fn test_export_character_empty() {
        let character = Character::default();
        let ui_canvas = Point::new(100.0, 100.0);

        let result = export_character(&character, &[AssetType::Face], (100, 100), ui_canvas);

        assert!(result.is_some());
        // Should be fully transparent
        let img = result.unwrap();
        for pixel in img.pixels() {
            assert_eq!(pixel[3], 0);
        }
    }

    #[test]
    fn test_export_character_with_part() {
        let mut character = Character::default();

        // Create a solid red square image
        let mut image = RgbaImage::new(10, 10);
        for pixel in image.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }

        let asset = Asset {
            id: "Test_Face".to_string(),
            name: "Test".to_string(),
            path: std::path::PathBuf::new(),
            back_part: None,
            asset_type: AssetType::Face,
            image_data: Some(Arc::new(image)),
        };

        let part = CharacterPart {
            position: Point::new(50.0, 50.0),
            scale: 1.0,
            rotation: 0.0,
            flipped: false,
            asset,
        };

        character.face = Some(part);

        let ui_canvas = Point::new(100.0, 100.0);

        let result = export_character(&character, &[AssetType::Face], (100, 100), ui_canvas);

        assert!(result.is_some());
        let img = result.unwrap();

        let centre_pixel = img.get_pixel(50, 50);
        assert_eq!(centre_pixel[0], 255);
        assert_eq!(centre_pixel[1], 0);
        assert_eq!(centre_pixel[2], 0);
        assert_eq!(centre_pixel[3], 255);
    }

    #[test]
    fn test_export_size_custom() {
        let custom = ExportSize::Custom {
            portrait: (120, 80),
            token: (50, 50),
        };
        assert_eq!(custom.portrait(), (120, 80));
        assert_eq!(custom.token(), (50, 50));
        assert_eq!(custom.display_name(), "Custom (120x80) (50x50)");
    }

    #[test]
    fn test_export_character_non_square() {
        let mut character = Character::default();

        let mut image = RgbaImage::new(10, 10);
        for pixel in image.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }

        let asset = Asset {
            id: "Test_Face".to_string(),
            name: "Test".to_string(),
            path: std::path::PathBuf::new(),
            back_part: None,
            asset_type: AssetType::Face,
            image_data: Some(Arc::new(image)),
        };

        // Centre on a 120x80 canvas
        let part = CharacterPart {
            position: Point::new(60.0, 40.0),
            scale: 1.0,
            rotation: 0.0,
            flipped: false,
            asset,
        };

        character.face = Some(part);

        let ui_canvas = Point::new(120.0, 80.0);
        let result = export_character(&character, &[AssetType::Face], (120, 80), ui_canvas);

        assert!(result.is_some());
        let img = result.unwrap();
        assert_eq!(img.width(), 120);
        assert_eq!(img.height(), 80);

        // Centre pixel (60, 40) should be red
        let centre_pixel = img.get_pixel(60, 40);
        assert_eq!(centre_pixel[0], 255);
        assert_eq!(centre_pixel[1], 0);
        assert_eq!(centre_pixel[2], 0);
        assert_eq!(centre_pixel[3], 255);
    }
}
