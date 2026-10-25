//! Color handling for the rendering backends.
//!
//! [`RaclettuiColor`] is an 8-bit RGBA color that ratatui [`Color`]s are
//! converted into before being drawn.

use ratatui_core::buffer::Cell;
use ratatui_core::style::{Color, Modifier};

/// An 8-bit RGBA color.
///
/// Created from a ratatui [`Color`] with [`From`], or directly from components:
///
/// ```
/// use raclettui::colors::RaclettuiColor;
/// use ratatui::style::Color;
///
/// let red = RaclettuiColor::from(Color::Rgb(255, 0, 0));
/// let translucent = RaclettuiColor::from_rgb(255, 0, 0).set_alpha(0.5);
/// assert_eq!(red.to_rgba(), (255, 0, 0, 255));
/// assert_eq!(translucent.to_rgba(), (255, 0, 0, 127));
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RaclettuiColor(u8, u8, u8, u8);

impl RaclettuiColor {
    /// Fully transparent black.
    pub fn new() -> Self {
        Self(0, 0, 0, 0)
    }

    /// Creates a color from red, green, blue and alpha components.
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }

    /// Creates an opaque color from red, green and blue components.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b, 255)
    }

    /// Returns the color with its alpha replaced by `alpha` (`0.0..=1.0`).
    pub fn set_alpha(self, alpha: f32) -> Self {
        let a = (alpha.clamp(0.0, 1.0) * 255.).floor() as u8;
        Self(self.0, self.1, self.2, a)
    }

    /// Converts to premultiplied linear RGBA, as used by the wgpu backend.
    pub fn to_linear(&self) -> [f32; 4] {
        let r = Self::linearise(self.0) as f32;
        let g = Self::linearise(self.1) as f32;
        let b = Self::linearise(self.2) as f32;
        let a = self.3 as f32 / 255.;
        [r * a, g * a, b * a, a]
    }

    fn linearise(value: u8) -> f64 {
        let v = value as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Returns the color with alpha `alpha` (`0.0..=1.0`) and the color
    /// components premultiplied by it, as expected by Wayland shm buffers.
    pub fn premultiply_alpha(self, alpha: f32) -> Self {
        let alpha = alpha.clamp(0.0, 1.0);
        let r = (self.0 as f32 * alpha).floor() as u8;
        let g = (self.1 as f32 * alpha).floor() as u8;
        let b = (self.2 as f32 * alpha).floor() as u8;
        let a = (alpha * 255.).floor() as u8;
        Self(r, g, b, a)
    }

    /// Returns the `(r, g, b, a)` components.
    pub fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (self.0, self.1, self.2, self.3)
    }

    /// Converts an xterm 256 color palette index into a `0xRRGGBB` value.
    pub fn indexed_color_to_rgb(index: u8) -> u32 {
        match index {
            // Basic 16 colors (0-15)
            0..=15 => {
                const BASIC_COLORS: [u32; 16] = [
                    0x000000, // 0: black
                    0xCD0000, // 1: red
                    0x00CD00, // 2: green
                    0xCDCD00, // 3: yellow
                    0x0000EE, // 4: blue
                    0xCD00CD, // 5: magenta
                    0x00CDCD, // 6: cyan
                    0xE5E5E5, // 7: white
                    0x7F7F7F, // 8: bright Black
                    0xFF0000, // 9: bright Red
                    0x00FF00, // 10: bright Green
                    0xFFFF00, // 11: bright Yellow
                    0x5C5CFF, // 12: bright Blue
                    0xFF00FF, // 13: bright Magenta
                    0x00FFFF, // 14: bright Cyan
                    0xFFFFFF, // 15: bright White
                ];
                BASIC_COLORS[index as usize]
            }

            // 216-color cube (16-231)
            16..=231 => {
                let cube_index = index - 16;
                let r = cube_index / 36;
                let g = (cube_index % 36) / 6;
                let b = cube_index % 6;

                // Convert 0-5 range to 0-255 RGB
                // Values: 0 -> 0, 1 -> 95, 2 -> 135, 3 -> 175, 4 -> 215, 5 -> 255
                let to_rgb = |n: u8| -> u32 {
                    if n == 0 {
                        0
                    } else {
                        55 + 40 * n as u32
                    }
                };

                (to_rgb(r) << 16) | (to_rgb(g) << 8) | to_rgb(b)
            }

            // 24 grayscale colors (232-255)
            232..=255 => {
                let gray_index = index - 232;
                // linear interpolation from 8 to 238
                let gray = (8 + gray_index * 10) as u32;
                (gray << 16) | (gray << 8) | gray
            }
        }
    }

    /// Default foreground, used for [`Color::Reset`] text.
    pub(crate) const DEFAULT_FG: Self = Self(255, 255, 255, 255);
    /// Default background, used for [`Color::Reset`] backgrounds and when clearing.
    pub(crate) const DEFAULT_BG: Self = Self(0, 0, 0, 255);

    /// Converts a ratatui color used as a background.
    ///
    /// Unlike [`From`], [`Color::Reset`] maps to the default background (black).
    pub(crate) fn from_bg(color: Color) -> Self {
        match color {
            Color::Reset => Self::DEFAULT_BG,
            color => Self::from(color),
        }
    }
}

/// Resolves the foreground and background colors of a cell, applying the
/// [`Modifier::REVERSED`] and [`Modifier::HIDDEN`] modifiers.
pub(crate) fn cell_colors(cell: &Cell) -> (RaclettuiColor, RaclettuiColor) {
    let mut fg = RaclettuiColor::from(cell.fg);
    let mut bg = RaclettuiColor::from_bg(cell.bg);
    if cell.modifier.contains(Modifier::REVERSED) {
        std::mem::swap(&mut fg, &mut bg);
    }
    if cell.modifier.contains(Modifier::HIDDEN) {
        fg = bg;
    }
    (fg, bg)
}

impl std::convert::From<ratatui_core::style::Color> for RaclettuiColor {
    /// Converts a ratatui color. [`Color::Reset`] maps to white, the default
    /// foreground color.
    fn from(value: ratatui_core::style::Color) -> Self {
        let a = 255;

        match value {
            Color::Rgb(r, g, b) => Self(r, g, b, a),
            Color::Black => Self(0, 0, 0, a),
            Color::Red => Self(128, 0, 0, a),
            Color::Green => Self(0, 128, 0, a),
            Color::Yellow => Self(128, 128, 0, a),
            Color::Blue => Self(0, 0, 128, a),
            Color::Magenta => Self(128, 0, 128, a),
            Color::Cyan => Self(0, 128, 128, a),
            Color::Gray => Self(192, 192, 192, a),
            Color::DarkGray => Self(128, 128, 128, a),
            Color::LightRed => Self(255, 0, 0, a),
            Color::LightGreen => Self(0, 255, 0, a),
            Color::LightYellow => Self(255, 255, 0, a),
            Color::LightBlue => Self(0, 0, 255, a),
            Color::LightMagenta => Self(255, 0, 255, a),
            Color::LightCyan => Self(0, 255, 255, a),
            Color::White => Self(255, 255, 255, a),
            Color::Indexed(code) => {
                let rgb = Self::indexed_color_to_rgb(code);
                Self((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, a)
            },
            Color::Reset => Self::DEFAULT_FG,
        }
    }

}

impl std::convert::From<RaclettuiColor> for glyphon::Color {
    fn from(color: RaclettuiColor) -> Self {
        glyphon::Color::rgba(color.0, color.1, color.2, color.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_colors() {
        assert_eq!(RaclettuiColor::from(Color::Indexed(1)).to_rgba(), (0xCD, 0, 0, 255));
        assert_eq!(RaclettuiColor::from(Color::Indexed(196)).to_rgba(), (255, 0, 0, 255));
        assert_eq!(RaclettuiColor::from(Color::Indexed(232)).to_rgba(), (8, 8, 8, 255));
        assert_eq!(RaclettuiColor::from(Color::Indexed(255)).to_rgba(), (238, 238, 238, 255));
    }

    #[test]
    fn reset_colors() {
        assert_eq!(RaclettuiColor::from(Color::Reset), RaclettuiColor::DEFAULT_FG);
        assert_eq!(RaclettuiColor::from_bg(Color::Reset), RaclettuiColor::DEFAULT_BG);
    }

    #[test]
    fn reversed_and_hidden_cells() {
        let mut cell = Cell::new("a");
        cell.fg = Color::Red;
        cell.bg = Color::Blue;
        cell.modifier = Modifier::REVERSED;
        let (fg, bg) = cell_colors(&cell);
        assert_eq!(fg, RaclettuiColor::from(Color::Blue));
        assert_eq!(bg, RaclettuiColor::from(Color::Red));

        cell.modifier = Modifier::HIDDEN;
        let (fg, bg) = cell_colors(&cell);
        assert_eq!(fg, bg);
    }

    #[test]
    fn alpha_is_clamped() {
        let c = RaclettuiColor::from_rgb(200, 100, 50);
        assert_eq!(c.set_alpha(2.0).to_rgba().3, 255);
        assert_eq!(c.premultiply_alpha(-1.0).to_rgba(), (0, 0, 0, 0));
        assert_eq!(c.premultiply_alpha(0.5).to_rgba(), (100, 50, 25, 127));
    }
}
