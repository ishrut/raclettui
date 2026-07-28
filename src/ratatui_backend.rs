use super::error::Error;
use super::window::LayerShellWindow;
use ratatui_core::backend::Backend;

use beamterm_core::{FontStyle, GlyphEffect};
use glutin::prelude::GlSurface;
use ratatui_core::style::Color;
use ratatui_core::style::Modifier;

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
            (0, 0, 0),
            (128, 0, 0),
            (0, 128, 0),
            (128, 128, 0),
            (0, 0, 128),
            (128, 0, 128),
            (0, 128, 128),
            (192, 192, 192),
            (128, 128, 128),
            (255, 0, 0),
            (0, 255, 0),
            (255, 255, 0),
            (0, 0, 255),
            (255, 0, 255),
            (0, 255, 255),
            (255, 255, 255),
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

impl Backend for LayerShellWindow {
    type Error = Error;
    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>,
    {
        loop {
            self.wayland_event_queue
                .blocking_dispatch(&mut self.wayland_state)
                .map_err(|e| Error::WaylandDispatchError(e))?;
            // implement timeout
            if self.wayland_state.needs_redraw {
                break;
            }
        }

        let cell_datas = content.map(|(x, y, cell)| {
            let cell_data = beamterm_core::CellData::new(
                cell.symbol(),
                cell.modifier.style(),
                cell.modifier.effect(),
                cell.fg.to_u32(),
                cell.bg.to_u32(),
            );
            (x, y, cell_data)
        });

        self.grid
            .update_cells_by_position(cell_datas)
            .map_err(|e| Error::BeamTermError(e))?;
        self.grid
            .flush_cells(&self.gl)
            .map_err(|e| Error::BeamTermError(e))?;

        self.gl_state.viewport(
            &self.gl,
            0,
            0,
            self.wayland_state.window_width as i32,
            self.wayland_state.window_height as i32,
        );

        self.grid
            .render(&self.gl, &mut self.gl_state)
            .map_err(|e| Error::BeamTermError(e))?;

        self.gl_surface
            .swap_buffers(&self.gl_context)
            .map_err(|e| Error::GlutinError(e))?;

        // Resets redraw callback
        self.set_redraw()
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        // self.grid
        //     .flush_cells(&self.gl)
        //     .map_err(|e| Error::BeamTermError(e))
        Ok(())
    }
    fn size(&self) -> Result<ratatui_core::layout::Size, Self::Error> {
        let width = self.grid.terminal_size().cols;
        let height = self.grid.terminal_size().rows;
        let size = ratatui_core::layout::Size::new(width, height);
        Ok(size)
    }
    fn window_size(&mut self) -> Result<ratatui_core::backend::WindowSize, Self::Error> {
        let grid_size = self.size()?;
        let pixel_size = ratatui_core::layout::Size::new(
            self.wayland_state.window_width as u16,
            self.wayland_state.window_height as u16,
        );
        let window_size = ratatui_core::backend::WindowSize {
            columns_rows: grid_size,
            pixels: pixel_size,
        };
        Ok(window_size)
    }
    fn clear(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn clear_region(
        &mut self,
        _clear_type: ratatui_core::backend::ClearType,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_cursor_position(&mut self) -> Result<ratatui_core::layout::Position, Self::Error> {
        let position = ratatui_core::layout::Position::new(0, 0);
        Ok(position)
    }
    fn get_cursor(&mut self) -> Result<(u16, u16), Self::Error> {
        Ok((0, 0))
    }
    fn append_lines(&mut self, _n: u16) -> Result<(), Self::Error> {
        Ok(())
    }
    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_cursor_position<P: Into<ratatui_core::layout::Position>>(
        &mut self,
        _position: P,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_cursor(&mut self, _x: u16, _y: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}
