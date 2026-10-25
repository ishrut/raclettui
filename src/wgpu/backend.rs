use ratatui_core::backend::{ClearType, WindowSize};
use ratatui_core::layout::{Position, Size};

use super::window::WgpuWindow;
use crate::colors::cell_colors;
use crate::Error;

impl ratatui_core::backend::Backend for WgpuWindow {

    type Error = crate::Error ;

    /// Updates the changed cells of the grid. The grid is rendered on `flush`.
    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
        where I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>
    {
        let grid = &mut self.grid_renderer.grid;
        for (x, y, cell) in content {
            let (x, y) = (x as u32, y as u32);
            // cells outside of the grid can show up for one frame after a resize
            if cell.skip || x >= grid.cols || y >= grid.rows {
                continue;
            }
            // wide characters leave the following cell with an empty symbol
            let ch = cell.symbol().chars().next().unwrap_or(' ');

            let (fg, bg) = cell_colors(cell);
            let bg = bg.set_alpha(grid.bg_alpha);
            let fg = fg.set_alpha(grid.fg_alpha);

            grid.set_bg(y, x, bg)?;
            grid.set_ch(y, x, ch, fg, cell.modifier, &mut self.grid_renderer.font_system)?;
        }
        Ok(())
    }

    /// Waits for the compositor to be ready for a new frame, then renders the
    /// grid. Pending input events are dispatched while waiting.
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.render()
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.grid_renderer.clear();
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        match clear_type {
            ClearType::All => self.clear(),
            _ => Err(Error::UnsupportedBackendFeature),
        }
    }

    fn size(&self) -> Result<Size, Self::Error> {
        let (cols, rows) = self.grid_size();
        Ok(Size::new(cols as u16, rows as u16))
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: Size::new(self.width() as u16, self.height() as u16),
        })
    }

    // there is no cursor to show, but ratatui calls these every frame so they must not fail
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        Ok(self.cursor)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        self.cursor = position.into();
        Ok(())
    }

    fn append_lines(&mut self, _n: u16) -> Result<(), Self::Error> {
        Err(Error::UnsupportedBackendFeature)
    }
}
