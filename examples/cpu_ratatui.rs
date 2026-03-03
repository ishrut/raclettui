use raclettui::{
    WindowBuilder,
    events::{KeyCode, WindowEvent},
    layer::{Anchor, Layer}
};

use ratatui::{
    Terminal,
    border,
    style::{Style, Styled},
    widgets::{Block, Paragraph}
};

fn main() {
    // let font = include_bytes!("../fonts/DaddyTimeMonoNerdFont-Regular.ttf") as &[u8];
    // let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

    let window = WindowBuilder::new()
        .set_namespace("example")
        .set_width(500)
        .set_height(484)
        .set_layer(Layer::Top)
        .set_anchor(Anchor::Top)
        .set_font_path("fonts/AdwaitaMonoNerdFont-Regular.ttf")
        .set_keyboard_interactivity(raclettui::KeyboardInteractivity::OnDemand)
        .bg_alpha(0.5)
        .init_cpu().unwrap();

    let events = window.get_event_queue();
    let mut terminal = Terminal::new(window).unwrap();

    'app_loop: loop{

        terminal.draw(|f| {
            let size = f.area();
            let paragraph = Paragraph::new("Hello World!")
                .block(
                    Block::new()
                    .borders(border!(TOP, BOTTOM, RIGHT, LEFT))
                )
                .set_style(Style::new().fg(ratatui::style::Color::Red).bg(ratatui::style::Color::Blue));
            f.render_widget(paragraph, size);
        }).unwrap();

        for ev in events.drain() {
            if let WindowEvent::Keyboard(key_event) = ev {
                match key_event.code {
                    KeyCode::Char('q') => break 'app_loop,
                    _ => {}
                }
            }
        }
    }
}


