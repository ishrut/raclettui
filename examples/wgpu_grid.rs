//! Draws a color gradient cell by cell with the wgpu backend, without ratatui.
//!
//! Run with: `cargo run --example wgpu_grid`
//! Exit with Ctrl+C.

use raclettui::{Anchor, Layer, WindowBuilder, colors::RaclettuiColor};

fn main() -> Result<(), raclettui::Error> {
    let mut window = WindowBuilder::new()
        .set_namespace("wgpu-grid")
        .set_width(300)
        .set_height(300)
        .set_layer(Layer::Top)
        .set_anchor(Anchor::Top)
        .set_font_size(18.)
        .set_bg_alpha(0.5)
        .init_wgpu()?;

    let (cols, rows) = window.grid_size();
    println!("cols: {cols}, rows: {rows}");

    for row in 0..rows {
        for col in 0..cols {
            // Character pattern (A-Z cycling)
            let ch = (b'A' + ((row * cols + col) % 26) as u8) as char;

            // Foreground color gradient
            let fg = RaclettuiColor::from_rgb(
                ((row * 40) % 256) as u8,
                ((col * 25) % 256) as u8,
                (((row + col) * 15) % 256) as u8,
            );

            // Background color gradient (different pattern)
            let bg = RaclettuiColor::from_rgb(
                ((col * 30) % 256) as u8,
                ((row * 50) % 256) as u8,
                (((row * col) * 5) % 256) as u8,
            );

            window.set_cell(col, row, ch, fg, bg)?;
        }
    }

    // render() waits for the compositor before every frame
    loop {
        window.render()?;
    }
}
