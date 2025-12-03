// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use egui::Color32;

/// Finds a contrasting colour.
pub(crate) trait Contrast {
    /// Finds a contrasting colour (either black or white) for the given colour.
    fn find_contrasting_colour(&self) -> Color32;
    /// Finds a contrasting colour, considering a background for transparency.
    fn find_contrasting_colour_on_background(&self, background: Color32) -> Self;
}

impl Contrast for Color32 {
    fn find_contrasting_colour(&self) -> Color32 {
        let foreground_r = self.r() as f32 / 255.0;
        let foreground_g = self.g() as f32 / 255.0;
        let foreground_b = self.b() as f32 / 255.0;
        let luminance = 0.2126 * foreground_r + 0.7152 * foreground_g + 0.0722 * foreground_b;
        if luminance > 0.5 {
            Self::BLACK
        } else {
            Self::WHITE
        }
    }

    fn find_contrasting_colour_on_background(&self, background: Color32) -> Self {
        let foreground_r = self.r() as f32 / 255.0;
        let foreground_g = self.g() as f32 / 255.0;
        let foreground_b = self.b() as f32 / 255.0;
        let foreground_a = self.a() as f32 / 255.0;
        let background_r = background.r() as f32 / 255.0;
        let background_g = background.g() as f32 / 255.0;
        let background_b = background.b() as f32 / 255.0;
        let r_final = foreground_r * foreground_a + background_r * (1.0 - foreground_a);
        let g_final = foreground_g * foreground_a + background_g * (1.0 - foreground_a);
        let b_final = foreground_b * foreground_a + background_b * (1.0 - foreground_a);
        let luminance = 0.2126 * r_final + 0.7152 * g_final + 0.0722 * b_final;
        if luminance > 0.5 {
            Self::BLACK
        } else {
            Self::WHITE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn test_find_contrasting_colour_bright() {
        // White
        assert_eq!(Color32::WHITE.find_contrasting_colour(), Color32::BLACK);
        // Yellow (high luminance)
        assert_eq!(Color32::YELLOW.find_contrasting_colour(), Color32::BLACK);
        // Light Grey
        assert_eq!(
            Color32::from_rgb(200, 200, 200).find_contrasting_colour(),
            Color32::BLACK
        );
    }

    #[test]
    fn test_find_contrasting_colour_dark() {
        // Black
        assert_eq!(Color32::BLACK.find_contrasting_colour(), Color32::WHITE);
        // Blue (low luminance)
        assert_eq!(Color32::BLUE.find_contrasting_colour(), Color32::WHITE);
        // Dark Grey
        assert_eq!(
            Color32::from_rgb(50, 50, 50).find_contrasting_colour(),
            Color32::WHITE
        );
    }

    #[test]
    fn test_find_contrasting_colour_on_background_transparent_bright_on_dark() {
        // A transparent white colour on a black background should effectively be a dark colour.
        let transparent_white = Color32::from_rgba_unmultiplied(255, 255, 255, 10); // Low alpha
        let background = Color32::BLACK; // Dark background
        // r_final = 1.0 * (10/255) + 0.0 * (1 - 10/255) = 10/255
        // Luminance will be very low (close to 0), so expect WHITE
        assert_eq!(
            transparent_white.find_contrasting_colour_on_background(background),
            Color32::WHITE
        );
    }

    #[test]
    fn test_find_contrasting_colour_on_background_transparent_dark_on_bright() {
        // A transparent black colour on a white background should effectively be a bright colour.
        let transparent_black = Color32::from_rgba_unmultiplied(0, 0, 0, 10); // Low alpha
        let background = Color32::WHITE; // Bright background
        // r_final = 0.0 * (10/255) + 1.0 * (1 - 10/255) = 245/255
        // Luminance will be very high (close to 1), so expect BLACK
        assert_eq!(
            transparent_black.find_contrasting_colour_on_background(background),
            Color32::BLACK
        );
    }

    #[test]
    fn test_find_contrasting_colour_on_background_opaque() {
        // Opaque white on any background should be black (luminance 1.0)
        let opaque_white = Color32::WHITE; // Alpha is 255
        let background = Color32::BLACK;
        assert_eq!(
            opaque_white.find_contrasting_colour_on_background(background),
            Color32::BLACK
        );

        // Opaque black on any background should be white (luminance 0.0)
        let opaque_black = Color32::BLACK; // Alpha is 255
        let background = Color32::WHITE;
        assert_eq!(
            opaque_black.find_contrasting_colour_on_background(background),
            Color32::WHITE
        );
    }
}
