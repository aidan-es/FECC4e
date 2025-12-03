// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::types::Rgba;

const COLOUR_ADJUSTMENT_FACTOR: f32 = 0.7;

pub trait AdjustBrightness {
    fn brighter(&self) -> Self;
    fn darker(&self) -> Self;
}

impl AdjustBrightness for Rgba {
    fn brighter(&self) -> Self {
        const MIN_BRIGHT: u8 = (1.0 / (1.0 - COLOUR_ADJUSTMENT_FACTOR)) as u8;

        // Special case 1: Rgba::BLACK.brighter() should return a dark grey.
        if self.r == 0 && self.g == 0 && self.b == 0 {
            return Self::new(MIN_BRIGHT, MIN_BRIGHT, MIN_BRIGHT, self.a);
        }

        let mut r = self.r;
        let mut g = self.g;
        let mut b = self.b;

        // Special case 2: Boost very dark colours.
        if r > 0 && r < MIN_BRIGHT {
            r = MIN_BRIGHT;
        }
        if g > 0 && g < MIN_BRIGHT {
            g = MIN_BRIGHT;
        }
        if b > 0 && b < MIN_BRIGHT {
            b = MIN_BRIGHT;
        }

        let new_r = (r as f32 / COLOUR_ADJUSTMENT_FACTOR).min(255.0) as u8;
        let new_g = (g as f32 / COLOUR_ADJUSTMENT_FACTOR).min(255.0) as u8;
        let new_b = (b as f32 / COLOUR_ADJUSTMENT_FACTOR).min(255.0) as u8;

        Self::new(new_r, new_g, new_b, self.a)
    }

    fn darker(&self) -> Self {
        let new_r = (self.r as f32 * COLOUR_ADJUSTMENT_FACTOR) as u8;
        let new_g = (self.g as f32 * COLOUR_ADJUSTMENT_FACTOR) as u8;
        let new_b = (self.b as f32 * COLOUR_ADJUSTMENT_FACTOR) as u8;

        Self::new(new_r, new_g, new_b, self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brighter() {
        let c = Rgba::new(82, 29, 255, 255);
        assert_eq!(c.brighter(), Rgba::new(117, 41, 255, 255));
    }

    #[test]
    fn test_brighter_clamping() {
        let c = Rgba::new(200, 200, 200, 255);
        // 200 / 0.7 = ~285, should clamp to 255
        assert_eq!(c.brighter(), Rgba::new(255, 255, 255, 255));
    }

    #[test]
    fn test_brighter_black() {
        let c = Rgba::BLACK;
        // Should return a dark grey, not black
        let brighter = c.brighter();
        assert!(brighter.r > 0);
        assert!(brighter.g > 0);
        assert!(brighter.b > 0);
        assert_eq!(brighter.a, 255);
    }

    #[test]
    fn test_darker() {
        let c = Rgba::new(45, 150, 139, 255);
        assert_eq!(c.darker(), Rgba::new(31, 105, 97, 255));
    }

    #[test]
    fn test_darker_zero() {
        let c = Rgba::new(0, 0, 0, 255);
        assert_eq!(c.darker(), Rgba::new(0, 0, 0, 255));
    }
}
