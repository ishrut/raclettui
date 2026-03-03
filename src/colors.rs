use ratatui_core::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct RaclettuiColor(u8, u8, u8, u8);

impl RaclettuiColor {
    pub fn new() -> Self {
        Self(0, 0, 0, 0)
    }
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b, 255)
    }
    pub fn set_alpha(self, alpha: f32) -> Self {
        let a = (alpha * 255.).floor() as u8;
        Self(self.0, self.1, self.2, a)
    }

    pub fn to_linear(&self) -> [f32; 4] {
        let r_linear = Self::linearise(self.0) as f32;
        let g_linear = Self::linearise(self.1) as f32;
        let b_linear = Self::linearise(self.2) as f32;
        let a_linear = self.3 as f32 / 255.;
        [r_linear, g_linear, b_linear, a_linear]
    }

    fn linearise(value: u8) -> f64 {
        let v = value as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn premultiply_alpha(self, alpha: f32) -> Self {
        let r = (self.0 as f32 * alpha).floor() as u8;
        let g = (self.1 as f32 * alpha).floor() as u8;
        let b = (self.2 as f32 * alpha).floor() as u8;
        let a = (alpha * 255.).floor() as u8;
        Self(r, g, b, a)
    }

    pub fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (self.0, self.1, self.2, self.3)
    }

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

                to_rgb(r) << 16 | to_rgb(g) << 8 | to_rgb(b)
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
}

impl std::convert::From<ratatui_core::style::Color> for RaclettuiColor {
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
                let hex = Self::indexed_color_to_rgb(code).to_ne_bytes();
                Self(hex[2], hex[1], hex[0], a)
            },
            Color::Reset => Self(255, 255, 255, a),
        }
    }

}

impl std::convert::Into<glyphon::Color> for RaclettuiColor {
    fn into(self) -> glyphon::Color {
        glyphon::Color::rgba(self.0, self.1, self.2, self.3)
    }
}
