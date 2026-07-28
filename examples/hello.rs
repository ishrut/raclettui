// use rewrite::events::{KeyCode, WindowEvent};
// use rewrite::layer_shell::{Anchor, KeyboardInteractivity, Layer};
use raclettui::{KeyCode, KeyboardInteractivity, WindowBuilder, WindowEvent};

use ratatui::{
    Terminal, border,
    style::{Color, Style, Styled},
    widgets::{Block, Paragraph},
};

fn main() {
    let window_builder = WindowBuilder {
        namespace: "example".to_string(),
        font_size: 16.,
        height: 500,
        width: 500,
        keyboard_interactivity: Some(KeyboardInteractivity::OnDemand),
        ..Default::default()
    };
    let window = window_builder.build().unwrap();

    let events = window.events();
    let mut terminal = Terminal::new(window).unwrap();

    loop {
        terminal
            .draw(|f| {
                let size = f.area();
                let paragraph = Paragraph::new("Hello  World")
                    .block(Block::new().borders(border!(TOP, BOTTOM, RIGHT, LEFT)))
                    .set_style(Style::new().fg(Color::Red).bg(Color::Blue));
                f.render_widget(paragraph, size);
            })
            .unwrap();

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
