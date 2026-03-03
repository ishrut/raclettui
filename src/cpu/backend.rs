use std::{
    io::{Seek, SeekFrom, Write},
};
use ratatui_core::backend::ClearType;

use super::window::CpuWindow;
use crate::colors::RaclettuiColor;
use crate::Error;

impl ratatui_core::backend::Backend for CpuWindow {
    type Error = Error;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>
    {
        loop {
            self.update()?;
            if self.state.needs_redraw {
                break
            }
        }
        self.state.needs_redraw = false;

        // getting variables
        let (file, surface) = match (&mut self.state.file, &self.state.surface) {
            (Some(file), Some(surface)) => (file, surface),
            _ => return Err(Error::WaylandSurfaceConfigurationError)
        };
        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::IoError(e))?;

        surface.damage(0, 0, self.state.window_width as i32, self.state.window_height as i32);

        for (x, y, cell) in content {

            let ch = cell.symbol().chars().next().unwrap_or(' ');

            let bg_col = RaclettuiColor::from(cell.bg).premultiply_alpha(self.alpha);
            let fg_col = RaclettuiColor::from(cell.fg).premultiply_alpha(self.alpha);

            self.buffer.set_bg_at_cell(x as u32, y as u32, &bg_col);
            self.buffer.set_char_at_cell(ch, x as u32, y as u32, &fg_col);
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {

        // getting variables
        let (file, surface, wl_buffer) =
            match (&mut self.state.file, &self.state.surface, &self.state.buffer) {
                (Some(file), Some(surface), Some(wl_buffer)) => (file, surface, wl_buffer),
                _ => return Err(Error::WaylandSurfaceConfigurationError)
            };

        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::IoError(e))?;
        file.write_all(&self.buffer.inner)
            .map_err(|e| Error::IoError(e))?;
        file.flush()
            .map_err(|e| Error::IoError(e))?;
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        self.state.frame_callback = Some(surface.frame(&self.event_queue.handle(), ()));

        Ok(())
    }

    fn size(&self) -> Result<ratatui_core::layout::Size, Self::Error> {
        let (width, height) = self.buffer.grid_dims();
        let size = ratatui_core::layout::Size::new(width as u16, height as u16);
        Ok(size)
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.buffer.clear(&RaclettuiColor::from_rgba(255, 255, 255, 255));
        Ok(())
    }

    fn get_cursor(&mut self) -> Result<(u16, u16), Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> Result<(), Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn window_size(&mut self) -> Result<ratatui_core::backend::WindowSize, Self::Error> {

        let (cell_width, cell_height) = self.buffer.grid_dims();
        let cell_size = ratatui_core::layout::Size::new(cell_width as u16, cell_height as u16);

        let(win_width, win_height) = self.buffer.window_size();
        let pixel_size = ratatui_core::layout::Size::new(win_width as u16, win_height as u16);
        let window_size = ratatui_core::backend::WindowSize {
            columns_rows: cell_size,
            pixels: pixel_size
        };
        Ok(window_size)
    }

    fn append_lines(&mut self, _n: u16) -> Result<(), Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }

    fn clear_region(&mut self, clear_type: ratatui_core::backend::ClearType) -> Result<(), Self::Error> {
        match clear_type {
            ClearType::All => self.clear()?,
            _ => return Err(Error::UnsupportedBackendFeature)
        }
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<ratatui_core::layout::Position, Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }

    fn set_cursor_position<P: Into<ratatui_core::layout::Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }
}


