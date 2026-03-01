use raclettui::{
    WindowBuilder,
    layer::{Anchor, Layer, KeyboardInteractivity},
    events::{WindowEvent, KeyCode},
};
use ratatui::{
    Terminal,
    border,
    style::{Style, Styled},
    widgets::{Block, Paragraph}
};

fn main(){

    let window = WindowBuilder::new()
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

    let events = window.get_event_queue();
    let mut terminal = Terminal::new(window).unwrap();

    'app_loop: loop{

        terminal.draw(|f| {
            let size = f.area();
            let paragraph = Paragraph::new("Hello World")
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
