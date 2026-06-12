use beamterm_core::CellData;
use ratatui_core::backend::ClearType;

use crate::utils::{RatatuiColorToU32, RatatuiModifierToStyleEffect};
use crate::window::Window;
use crate::Error;
use glutin::prelude::GlSurface;

impl ratatui_core::backend::Backend for Window {
    type Error = Error;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>,
    {
        // Only draw when wayland ready
        loop {
            self.update_states()?;
            if self.is_redraw() {
                break;
            }
        }

        // Converts the iterator for beamterm_core
        let cell_datas = content.map(|(x, y, cell)| {
            let cell_data = CellData::new(
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
            .render(&self.gl, &mut self.gl_state)
            .map_err(|e| Error::BeamTermError(e))?;

        self.gl_surface
            .swap_buffers(&self.gl_context)
            .map_err(|e| Error::GlutinError(e))?;

        // Resets redraw callback
        self.reset_draw()?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.grid.flush_cells(&self.gl).unwrap();
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn clear_region(
        &mut self,
        clear_type: ratatui_core::backend::ClearType,
    ) -> Result<(), Self::Error> {
        match clear_type {
            ClearType::All => self.clear(),
            _ => Ok(())

        }
    }

    fn size(&self) -> Result<ratatui_core::layout::Size, Self::Error> {
        let width = self.grid.terminal_size().cols;
        let height = self.grid.terminal_size().rows;
        let size = ratatui_core::layout::Size::new(width, height);
        Ok(size)
    }

    fn window_size(&mut self) -> Result<ratatui_core::backend::WindowSize, Self::Error> {
        let grid_size = self.size()?;
        let pixel_size = ratatui_core::layout::Size::new(self.width() as u16, self.height() as u16);
        let window_size = ratatui_core::backend::WindowSize {
            columns_rows: grid_size,
            pixels: pixel_size,
        };
        Ok(window_size)
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_cursor(&mut self) -> Result<(u16, u16), Self::Error> {
        Ok((0, 0))
    }

    fn get_cursor_position(&mut self) -> Result<ratatui_core::layout::Position, Self::Error> {
        let position = ratatui_core::layout::Position::new(0, 0);
        Ok(position)
    }

    fn set_cursor_position<P: Into<ratatui_core::layout::Position>>(
        &mut self,
        _position: P,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn append_lines(&mut self, _n: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}
