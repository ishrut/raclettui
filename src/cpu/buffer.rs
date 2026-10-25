use std::collections::HashMap;
use fontdue::{LineMetrics, Metrics};

use crate::colors::RaclettuiColor;

/// A software framebuffer laid out as a grid of terminal cells.
///
/// Pixels are stored as premultiplied ARGB8888 in little endian byte order
/// (`[b, g, r, a]`), which is the format of the Wayland shm buffer the
/// [`CpuWindow`](crate::CpuWindow) presents. Glyphs are rasterized with
/// `fontdue` and cached per character.
#[derive(Debug)]
pub struct TerminalBuffer {
    /// Raw pixel data, `window_width * window_height * 4` bytes.
    pub inner: Vec<u8>,
    /// Window width in pixels.
    pub window_width: u32,
    /// Window height in pixels.
    pub window_height: u32,
    /// Font used to rasterize glyphs.
    pub font: fontdue::Font,
    /// Font size in pixels.
    pub font_size: f32,
    /// Width of a cell in pixels.
    pub cell_width: u32,
    /// Height of a cell in pixels.
    pub cell_height: u32,
    /// Rasterized glyph cache.
    pub cache: HashMap<char, (Metrics, Vec<u8>)>,
    /// Horizontal line metrics of the font at `font_size`.
    pub line_metrics: LineMetrics,
}

impl TerminalBuffer {
    /// Creates a transparent buffer of `window_width` x `window_height` pixels.
    ///
    /// The cell size is derived from the advance width of `M` and the line
    /// height of `font` at `font_size`.
    pub fn new(
        window_width: u32,
        window_height: u32,
        font: fontdue::Font,
        font_size: f32,
    ) -> Self
    {
        let m_metrics = font.metrics('M', font_size);
        // fonts without horizontal metrics get a typical ascent/descent split
        let line_metrics = font.horizontal_line_metrics(font_size).unwrap_or(LineMetrics {
            ascent: font_size * 0.8,
            descent: -font_size * 0.2,
            line_gap: 0.0,
            new_line_size: font_size,
        });
        let cell_height = (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap)
            .ceil()
            .max(1.0) as u32;
        // the advance width is the distance between two characters, the
        // bitmap width of 'M' is narrower and makes glyphs overlap
        let cell_width = m_metrics.advance_width.ceil().max(1.0) as u32;

        Self {
            inner: vec![0u8; (window_width * window_height * 4) as usize],
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

    /// Number of `(columns, rows)` that fit in the window.
    pub fn grid_dims(&self) -> (u32, u32) {
        let cols = self.window_width / self.cell_width;
        let rows = self.window_height / self.cell_height;
        (cols, rows)
    }

    /// Window size in pixels as `(width, height)`.
    pub fn window_size(&self) -> (u32, u32) {
        (self.window_width, self.window_height)
    }

    /// Resizes the buffer to `window_width` x `window_height` pixels.
    /// The content is discarded and the buffer becomes transparent.
    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        self.window_width = window_width;
        self.window_height = window_height;
        self.inner = vec![0u8; (window_width * window_height * 4) as usize];
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: &RaclettuiColor) {
        if x >= self.window_width || y >= self.window_height {
            return;
        }
        let (r, g, b, a) = color.to_rgba();
        let index = ((y * self.window_width + x) * 4) as usize;
        self.inner[index..index + 4].copy_from_slice(&[b, g, r, a]);
    }

    /// Fills the whole buffer with a (premultiplied) color.
    pub fn clear(&mut self, color: &RaclettuiColor) {
        let (r, g, b, a) = color.to_rgba();
        for pixel in self.inner.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[b, g, r, a]);
        }
    }

    /// Fills the cell at column `x`, row `y` with a (premultiplied) color.
    /// Cells outside of the window are ignored.
    pub fn set_bg_at_cell(&mut self, x: u32, y: u32, color: &RaclettuiColor) {
        let cell_px_x = x * self.cell_width;
        let cell_px_y = y * self.cell_height;

        for i in 0..self.cell_height {
            for j in 0..self.cell_width {
                self.set_pixel(j + cell_px_x, i + cell_px_y, color);
            }
        }
    }

    /// Draws a character at column `x`, row `y`, blending it over the
    /// current content. The background should be drawn first with
    /// [`set_bg_at_cell`](Self::set_bg_at_cell).
    ///
    /// `color` is a premultiplied color, see
    /// [`RaclettuiColor::premultiply_alpha`].
    pub fn set_char_at_cell(&mut self, ch: char, x: u32, y: u32, color: &RaclettuiColor) {
        if ch == ' ' {
            return;
        }
        let (fr, fg, fb, fa) = color.to_rgba();

        let font = &self.font;
        let font_size = self.font_size;
        let (metrics, bitmap) = self
            .cache
            .entry(ch)
            .or_insert_with(|| font.rasterize(ch, font_size));

        let cell_px_x = x * self.cell_width;
        let cell_px_y = y * self.cell_height;

        let baseline_offset = (self.line_metrics.ascent).ceil() as i32;

        let glyph_x = cell_px_x as i32 + metrics.xmin;
        let glyph_y = cell_px_y as i32 + baseline_offset - metrics.height as i32 - metrics.ymin;

        for gy in 0..metrics.height {
            for gx in 0..metrics.width {
                let coverage = bitmap[gy * metrics.width + gx];
                if coverage == 0 {
                    continue;
                }

                let px = glyph_x + gx as i32;
                let py = glyph_y + gy as i32;

                if px < 0 ||
                    py < 0 ||
                    px >= self.window_width as i32 ||
                    py >= self.window_height as i32
                {
                    continue;
                }

                // premultiplied "source over" blending of the glyph onto the pixel
                let coverage = coverage as f32 / 255.0;
                let src = [fb, fg, fr, fa].map(|c| c as f32 * coverage);
                let inv_alpha = 1.0 - src[3] / 255.0;

                let index = ((py as u32 * self.window_width + px as u32) * 4) as usize;
                let pixel = &mut self.inner[index..index + 4];
                for (dst, src) in pixel.iter_mut().zip(src) {
                    *dst = (src + *dst as f32 * inv_alpha).round().min(255.0) as u8;
                }
            }
        }
    }
}
