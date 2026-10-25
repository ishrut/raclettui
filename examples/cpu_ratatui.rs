//! A minimal ratatui app using the CPU backend.
//!
//! Run with: `cargo run --example cpu_ratatui`
//! Set `RACLETTUI_FONT=/path/to/font.ttf` to use a specific font.
//! Press `q` or `Esc` to quit.

use raclettui::{
    WindowBuilder,
    events::{KeyCode, WindowEvent},
    layer::{Anchor, KeyboardInteractivity, Layer},
};
use ratatui::{
    Terminal,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

fn main() -> Result<(), raclettui::Error> {
    let mut builder = WindowBuilder::new()
        .set_namespace("cpu-example")
        .set_width(500)
        .set_height(300)
        .set_layer(Layer::Top)
        .set_anchor(Anchor::Top)
        .set_margin(20, 0, 0, 0)
        .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
        .set_bg_alpha(0.8);
    if let Ok(font) = std::env::var("RACLETTUI_FONT") {
        builder = builder.set_font_path(font);
    }

    let window = builder.init_cpu()?;
    let events = window.get_event_queue();
    let mut terminal = Terminal::new(window)?;

    let mut presses = 0;
    loop {
        terminal.draw(|f| {
            let text = format!("Hello World!\n\nkeys pressed: {presses}\n\npress q to quit");
            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::bordered().title(" raclettui · cpu "))
                .style(Style::new().fg(Color::Red).bg(Color::Blue));
            f.render_widget(paragraph, f.area());
        })?;

        for event in events.drain() {
            match event {
                WindowEvent::Keyboard(key) if key.is_press() => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => presses += 1,
                },
                WindowEvent::Closed => return Ok(()),
                _ => {}
            }
        }
    }
}
