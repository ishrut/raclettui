use raclettui::{Anchor, KeyboardInteractivity, Layer, WindowBuilder, colors::RaclettuiColor};

fn main() {
    let mut window = WindowBuilder::new()
        .set_namespace("example")
        .set_width(300)
        .set_height(300)
        .set_layer(Layer::Top)
        .set_anchor(Anchor::Top)
        .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
        // .set_font_path("fonts/Some-Mono-Font.ttf")
        .set_font_size(18.)
        .bg_alpha(0.5)
        // .init_cpu() // for cpu rendering
        .init_wgpu().unwrap();

    println!("rows: {}, cols: {}", window.grid_renderer.grid.rows, window.grid_renderer.grid.cols);

    // window.grid_renderer.grid.set_bg(0, 0, (255, 0, 0));
    // window.grid_renderer.grid.set_ch(0, 0, 'A', (255, 255, 255), &mut window.grid_renderer.font_system);
    let rows = window.grid_renderer.grid.rows;
    let cols = window.grid_renderer.grid.cols;

    for row in 0..rows {
        for col in 0..cols {

            // Character pattern (A-Z cycling)
            let ch = (b'A' + ((row * cols + col) % 26) as u8) as char;

            // Foreground color gradient
            let fg_r = ((row * 40) % 256) as u8;
            let fg_g = ((col * 25) % 256) as u8;
            let fg_b = (((row + col) * 15) % 256) as u8;
            let fg_color = RaclettuiColor::from_rgb(fg_r, fg_g, fg_b);

            // Background color gradient (different pattern)
            let bg_r = ((col * 30) % 256) as u8;
            let bg_g = ((row * 50) % 256) as u8;
            let bg_b = (((row * col) * 5) % 256) as u8;
            let bg_color = RaclettuiColor::from_rgb(bg_r, bg_g, bg_b);

            window.grid_renderer.grid.set_bg(row, col, bg_color);
            window.grid_renderer.grid.set_ch(
                row,
                col,
                ch,
                fg_color,
                &mut window.grid_renderer.font_system,
            );
        }
    }
    'app_loop: loop {
        window.update().unwrap();
        if !window.wayland_state.needs_redraw {
            continue;
        }
        window.wayland_state.needs_redraw = false;
        window.wayland_state.set_frame_callback(&window.wayland_event_queue.handle()).unwrap();
        window.render();



    }
}
