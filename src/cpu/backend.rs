use super::window::CpuWindow;
use crate::colors;

use crate::Error;
use std::io::Write;

impl ratatui_core::backend::Backend for CpuWindow {

    type Error = Error ;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
        where I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>
    {

        loop {
            self.update();
            if self.wl_state.is_redraw() {
                break
            }
        }
        self.wl_state.needs_redraw = false;

        let (file, surface, wl_buffer) =
            match (&mut self.wl_state.file, &self.wl_state.surface, &self.wl_state.buffer) {
                (Some(file), Some(surface), Some(wl_buffer)) => (file, surface, wl_buffer),
                _ => return Err(Error::WaylandSurfaceConfigurationError)
        };

        // notify to redraw whole surface
        surface.damage(0, 0, self.wl_state.window_width as i32, self.wl_state.window_height as i32);

        // draws cells
        for (x, y, cell) in content {
            let ch = cell.symbol().chars().next()
                .ok_or(Error::RatatuiBackendError)?;
            let bg = colors::to_rgba(cell.bg, (0, 0, 0), self.bg_alpha);
            let fg = colors::to_rgba(cell.fg,(255, 255, 255), self.fg_alpha);
            self.buffer.set_bg_at_cell(x as u32, y as u32, bg);
            self.buffer.set_char_at_cell(ch, x as u32, y as u32, fg);

        }

        // committing the changes
        file.write_all(&self.buffer.inner)
            .map_err(|e| Error::IoError(e))?;
        file.flush()
            .map_err(|e| Error::IoError(e))?;
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        // resetting callback
        self.wl_state.set_frame_callback(&self.wl_event_queue.handle())?;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn clear(&mut self) -> Result<(), Self::Error> {
        let color = colors::rgba_premultiplied((255, 255, 255, 255), self.bg_alpha);
        self.buffer.clear(color);
        Ok(())
    }
    fn clear_region(&mut self, clear_type: ratatui_core::backend::ClearType) -> Result<(), Self::Error> {
        match clear_type {
            ratatui_core::backend::ClearType::All => self.clear()?,
            _ => {}
        }
        Ok(())
    }
    fn size(&self) -> Result<ratatui_core::layout::Size, Self::Error> {
        let size = ratatui_core::layout::Size::new(self.cols() as u16, self.rows() as u16);
        Ok(size)
    }
    fn window_size(&mut self) -> Result<ratatui_core::backend::WindowSize, Self::Error> {
        let grid_size = self.size()?;
        let pixel_size = ratatui_core::layout::Size::new(self.width() as u16, self.height() as u16);
        let window_size = ratatui_core::backend::WindowSize {
            columns_rows: grid_size,
            pixels: pixel_size
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
    fn set_cursor_position<P: Into<ratatui_core::layout::Position>>(&mut self, _position: P) -> Result<(), Self::Error> {
        Ok(())
    }
    fn append_lines(&mut self, _n: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}
