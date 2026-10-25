//! A status bar spanning the top edge of the screen.
//!
//! The width is left at 0 so the compositor sizes the bar to the output,
//! and the exclusive zone keeps other windows from covering it.
//!
//! Run with: `cargo run --example bar`
//! Exit with Ctrl+C.

use std::time::{SystemTime, UNIX_EPOCH};

use raclettui::{
    WindowBuilder,
    events::WindowEvent,
    layer::{Anchor, Layer},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::Paragraph,
};

const HEIGHT: u32 = 24;

fn main() -> Result<(), raclettui::Error> {
    let window = WindowBuilder::new()
        .set_namespace("bar")
        .set_height(HEIGHT)
        .set_anchors(vec![Anchor::Top, Anchor::Left, Anchor::Right])
        .set_exclusive_zone(HEIGHT as i32)
        .set_layer(Layer::Top)
        .set_font_size(16.)
        .set_bg_alpha(0.9)
        .init_cpu()?;

    let events = window.get_event_queue();
    let mut terminal = Terminal::new(window)?;
    let mut resizes = 0;

    loop {
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let clock = format!("{:02}:{:02}:{:02} UTC", secs / 3600 % 24, secs / 60 % 60, secs % 60);

        terminal.draw(|f| {
            let [left, right] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(f.area());

            let workspaces = Line::from(vec![
                " 1 ".black().on_yellow(),
                " 2 ".into(),
                " 3 ".into(),
                format!("  resized {resizes}x").dark_gray(),
            ]);
            f.render_widget(Paragraph::new(workspaces), left);
            f.render_widget(Paragraph::new(clock).style(Style::new().fg(Color::Cyan)).right_aligned(), right);
        })?;

        for event in events.drain() {
            match event {
                WindowEvent::Resize { .. } => resizes += 1,
                WindowEvent::Closed => return Ok(()),
                _ => {}
            }
        }
    }
}
