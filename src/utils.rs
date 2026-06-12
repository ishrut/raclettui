use beamterm_core::{FontStyle, GlyphEffect};
use ratatui_core::style::Modifier;
use ratatui_core::style::Color;

pub trait RatatuiColorToU32 {
    fn to_u32(&self) -> u32;
}

impl RatatuiColorToU32 for Color {
    fn to_u32(&self) -> u32 {
        let (r, g, b) = match self {
            Color::Rgb(r, g, b) => (*r, *g, *b),
            Color::Indexed(i) => ansi256_to_rgb(*i),
            Color::Black => (0, 0, 0),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Yellow => (255, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Magenta => (255, 0, 255),
            Color::Cyan => (0, 255, 255),
            Color::Gray => (128, 128, 128),
            Color::DarkGray => (64, 64, 64),
            Color::LightRed => (255, 85, 85),
            Color::LightGreen => (85, 255, 85),
            Color::LightYellow => (255, 255, 85),
            Color::LightBlue => (85, 85, 255),
            Color::LightMagenta => (255, 85, 255),
            Color::LightCyan => (85, 255, 255),
            Color::White => (255, 255, 255),

            Color::Reset => (0, 0, 0), // or choose your default background
        };
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

fn ansi256_to_rgb(i: u8) -> (u8, u8, u8) {
    if i < 16 {
        const ANSI: [(u8, u8, u8); 16] = [
            (0, 0, 0),       (128, 0, 0),   (0, 128, 0),   (128, 128, 0),
            (0, 0, 128),     (128, 0, 128), (0, 128, 128), (192, 192, 192),
            (128, 128, 128), (255, 0, 0),   (0, 255, 0),   (255, 255, 0),
            (0, 0, 255),     (255, 0, 255), (0, 255, 255), (255, 255, 255),
        ];
        ANSI[i as usize]
    } else if i < 232 {
        let i = i - 16;
        let r = (i / 36) * 51;
        let g = ((i % 36) / 6) * 51;
        let b = (i % 6) * 51;
        (r, g, b)
    } else {
        let gray = 8 + (i - 232) * 10;
        (gray, gray, gray)
    }
}

pub trait RatatuiModifierToStyleEffect {
    fn style(&self) -> FontStyle;
    fn effect(&self) -> GlyphEffect;
}

impl RatatuiModifierToStyleEffect for Modifier {
    fn style(&self) -> FontStyle {
        if self.contains(Modifier::ITALIC) && self.contains(Modifier::BOLD) {
            FontStyle::BoldItalic
        } else if self.contains(Modifier::ITALIC) {
            FontStyle::Italic
        } else if self.contains(Modifier::BOLD) {
            FontStyle::Bold
        } else {
            FontStyle::Normal
        }
    }
    fn effect(&self) -> GlyphEffect {
        if self.contains(Modifier::CROSSED_OUT) {
            GlyphEffect::Strikethrough
        } else if self.contains(Modifier::UNDERLINED) {
            GlyphEffect::Underline
        } else {
            GlyphEffect::None
        }
    }
}
