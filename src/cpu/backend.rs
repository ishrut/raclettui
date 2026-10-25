use ratatui_core::backend::{ClearType, WindowSize};
use ratatui_core::layout::{Position, Size};

use super::window::CpuWindow;
use crate::colors::cell_colors;
use crate::Error;

impl ratatui_core::backend::Backend for CpuWindow {
    type Error = Error;

    /// Paints the changed cells into the pixel buffer. The buffer is presented on `flush`.
    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>
    {
        for (x, y, cell) in content {
            if cell.skip {
                continue;
            }
            let ch = cell.symbol().chars().next().unwrap_or(' ');

            let (fg, bg) = cell_colors(cell);
            let bg = bg.premultiply_alpha(self.bg_alpha);
            let fg = fg.premultiply_alpha(self.fg_alpha);

            self.buffer.set_bg_at_cell(x as u32, y as u32, &bg);
            self.buffer.set_char_at_cell(ch, x as u32, y as u32, &fg);
        }

        Ok(())
    }

    /// Waits for the compositor to be ready for a new frame, then presents the
    /// pixel buffer. Pending input events are dispatched while waiting.
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.present()
    }

    fn size(&self) -> Result<Size, Self::Error> {
        let (cols, rows) = self.buffer.grid_dims();
        Ok(Size::new(cols as u16, rows as u16))
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        let window_bg = self.window_bg();
        self.buffer.clear(&window_bg);
        Ok(())
    }

    // there is no cursor to show, but ratatui calls these every frame so they must not fail
    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        Ok(self.cursor)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        self.cursor = position.into();
        Ok(())
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        let (win_width, win_height) = self.buffer.window_size();
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: Size::new(win_width as u16, win_height as u16),
        })
    }

    fn append_lines(&mut self, _n: u16) -> Result<(), Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        match clear_type {
            ClearType::All => self.clear(),
            _ => Err(Error::UnsupportedBackendFeature),
        }
    }
}
