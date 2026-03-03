use super::window::WgpuWindow;
use crate::Error;
use crate::colors::RaclettuiColor;

impl ratatui_core::backend::Backend for WgpuWindow {

    type Error = crate::Error ;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
        where I: Iterator<Item = (u16, u16, &'a ratatui_core::buffer::Cell)>
    {

        loop {
            // update is blocking
            self.update()?;
            if self.wayland_state.is_redraw() {
                break
            }
        }
        self.wayland_state.needs_redraw = false;

        for (x, y, cell) in content {
            let ch = cell
                .symbol()
                .chars()
                .next()
                .ok_or(Error::RatatuiBackendError)?;

            let bg_alpha = self.grid_renderer.grid.bg_alpha;
            let fg_alpha = self.grid_renderer.grid.bg_alpha;
            let bg_color = RaclettuiColor::from(cell.bg).set_alpha(bg_alpha);
            let fg_color = RaclettuiColor::from(cell.fg).set_alpha(fg_alpha);

            self.grid_renderer.grid.set_bg(
                y as u32,
                x as u32,
                bg_color
            )?;
            self.grid_renderer.grid.set_ch(
                y as u32,
                x as u32,
                ch,
                fg_color,
                &mut self.grid_renderer.font_system,
            )?;
        }

        let output = self.wgpu_surface.get_current_texture()
            .map_err(|e| Error::WgpuSurfaceError(e))?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear({
                            wgpu::Color { r: 0., g: 0., b: 0., a: self.grid_renderer.grid.bg_alpha as f64 }
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.grid_renderer.render_background(&self.wgpu_device, &mut render_pass);
            self.grid_renderer.render_text(&mut self.wgpu_queue, &self.wgpu_device, &mut render_pass)?;
        }
        self.wgpu_queue.submit(vec![encoder.finish()]);
        output.present();

        // resetting callback
        self.wayland_state.set_frame_callback(&self.wayland_event_queue.handle())?;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn clear(&mut self) -> Result<(), Self::Error> {
        self.clear_screen()?;
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
        let width = self.grid_renderer.grid.cols;
        let height = self.grid_renderer.grid.rows;
        let size = ratatui_core::layout::Size::new(width as u16, height as u16);
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

