// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        let len = hex.len();
        if len == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            Ok(Self::new(r, g, b, 255))
        } else if len == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
            Ok(Self::new(r, g, b, a))
        } else {
            Err(format!("Invalid hex length: {}", len))
        }
    }
}

impl From<[u8; 4]> for Rgba {
    fn from(arr: [u8; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
}

impl From<Rgba> for [u8; 4] {
    fn from(c: Rgba) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self::new(0.0, 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_from_hex_valid_6() {
        let colour = Rgba::from_hex("#FF0000").unwrap();
        assert_eq!(colour, Rgba::new(255, 0, 0, 255));

        let colour = Rgba::from_hex("00FF00").unwrap();
        assert_eq!(colour, Rgba::new(0, 255, 0, 255));
    }

    #[test]
    fn test_rgba_from_hex_valid_8() {
        let colour = Rgba::from_hex("#0000FF80").unwrap();
        assert_eq!(colour, Rgba::new(0, 0, 255, 128));
    }

    #[test]
    fn test_rgba_from_hex_invalid() {
        assert!(Rgba::from_hex("#ZZZZZZ").is_err()); // Invalid chars
        assert!(Rgba::from_hex("#123").is_err()); // Invalid length
        assert!(Rgba::from_hex("").is_err()); // Empty
    }

    #[test]
    fn test_rgba_from_into_array() {
        let arr = [10, 20, 30, 40];
        let colour: Rgba = arr.into();
        assert_eq!(colour, Rgba::new(10, 20, 30, 40));

        let arr2: [u8; 4] = colour.into();
        assert_eq!(arr2, arr);
    }
}
