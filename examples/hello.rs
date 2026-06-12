use raclettui::events::{KeyCode, WindowEvent};
use raclettui::{Anchor, KeyboardInteractivity, Layer, WindowBuilder};

use ratatui::{
    Terminal,
    border,
    style::{Style, Styled, Color},
    widgets::{Block, Paragraph}
};

fn main() {
    env_logger::init();

    let window = WindowBuilder::new()
        .set_namespace("example")
        .set_width(300)
        .set_height(300)
        .set_layer(Layer::Overlay)
        .set_anchor(Anchor::Top)
        .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
        .set_bg_alpha(0.5)
        .init()
        .unwrap();

    let events = window.events();
    let mut terminal = Terminal::new(window).unwrap();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let paragraph = Paragraph::new("Hello World")
                .block(
                    Block::new()
                    .borders(border!(TOP, BOTTOM, RIGHT, LEFT))
                )
                .set_style(
                    Style::new()
                        .fg(Color::Red)
                        .bg(Color::Blue)
                );
            f.render_widget(paragraph, size);
        }).unwrap();

        for ev in events.drain() {
            if let WindowEvent::Keyboard(key_event) = ev {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => return,
                    _ => {}
                }
            }
        }
    }
}
