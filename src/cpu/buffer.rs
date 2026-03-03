use std::collections::HashMap;
use fontdue::{LineMetrics, Metrics};

use crate::colors::RaclettuiColor;

#[derive(Debug)]
pub struct TerminalBuffer {
    pub inner: Vec<u8>,
    // window width in px
    pub window_width: u32,
    // window height in px
    pub window_height: u32,
    pub font: fontdue::Font,
    // in px
    pub font_size: f32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub cache: HashMap<char, (Metrics, Vec<u8>)>,
    pub line_metrics: LineMetrics,
}

impl TerminalBuffer {
    pub fn new(
        window_width: u32,
        window_height: u32,
        font: fontdue::Font,
        font_size: f32,
    ) -> Self
    {
        let m_metrics = font.metrics('M', font_size);
        let line_metrics = font.horizontal_line_metrics(font_size).unwrap();
        let cell_height = (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap).ceil() as u32;
        let cell_width = m_metrics.width as u32;

        let inner = vec![0u8; (window_width * window_height * 4) as usize];

        Self {
            inner,
            window_width,
            window_height,
            font,
            font_size,
            cell_width,
            cell_height,
            cache: HashMap::<char, (Metrics, Vec<u8>)>::new(),
            line_metrics
        }
    }

    pub fn grid_dims(&self) -> (u32, u32) {
        let cols = self.window_width / self.cell_width;
        let rows = self.window_height / self.cell_height;
        (cols, rows)
    }

    pub fn window_size(&self) -> (u32, u32) {
        (self.window_width, self.window_height)
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: &RaclettuiColor) {
        if x >= self.window_width {
            panic!("x out of range x: {}, y: {}", x, y)
        }
        if y >= self.window_height {
            panic!("y out of range x: {}, y: {}", x, y)
        }
        let value = color.to_rgba();

        let index = ((y * self.window_width + x) * 4) as usize;
        self.inner[index] = value.2;
        self.inner[index + 1] = value.1;
        self.inner[index + 2] = value.0;
        self.inner[index + 3] = value.3;
    }

    // clears whole buffer with a color
    pub fn clear(&mut self, color: &RaclettuiColor) {
        for i in 0..self.window_height {
            for j in 0..self.window_width {
                self.set_pixel(j, i, color);
            }
        }
    }

    // sets background at a cell
    pub fn set_bg_at_cell(&mut self, x: u32, y: u32, color: &RaclettuiColor) {
        let cell_px_x = x * self.cell_width;
        let cell_px_y = y * self.cell_height;

        for i in 0..self.cell_height {
            for j in 0..self.cell_width {
                self.set_pixel(j + cell_px_x, i + cell_px_y, color);
            }
        }
    }

    // draws a character at a cell, not that background needs prior setting
    pub fn set_char_at_cell(&mut self, ch: char, x: u32, y: u32, color: &RaclettuiColor) {
        let color = color.to_rgba();
        if !self.cache.contains_key(&ch) {
            let val = self.font.rasterize(ch, self.font_size);
            self.cache.insert(ch, val);
        }
        let (metrics, bitmap) = &self.cache.get(&ch).unwrap().clone();


        let cell_px_x = x * self.cell_width;
        let cell_px_y = y * self.cell_height;

        let baseline_offset = (self.line_metrics.ascent).ceil() as i32;

        let glyph_x = cell_px_x as i32 + metrics.xmin;
        let glyph_y = cell_px_y as i32 + baseline_offset - metrics.height as i32 - metrics.ymin;
        let metrics_height = metrics.height;
        let metrics_width = metrics.width;

        for y in 0..metrics_height {
            for x in 0..metrics_width {
                let alpha = bitmap[y * metrics_width + x];
                if alpha == 0 {
                    continue;
                }

                let px = glyph_x + x as i32;
                let py = glyph_y + y as i32;

                if px < 0 ||
                    py < 0 ||
                    px >= self.window_width as i32 ||
                    py >= self.window_height as i32
                {
                    continue;
                }

                let alpha_f = alpha as f32 / 255.0;

                let r = (color.0 as f32 * alpha_f) as u8;
                let g = (color.1 as f32 * alpha_f) as u8;
                let b = (color.2 as f32 * alpha_f) as u8;

                self.set_pixel(
                    px as u32,
                    py as u32,
                    &RaclettuiColor::from_rgb(r, g, b),
                );

            }
        }
    }
}
