//! A launcher-style menu centered on the screen, navigated with the keyboard
//! or the mouse wheel. Prints the selected entry and exits.
//!
//! Run with: `cargo run --example menu` (CPU) or `cargo run --example menu -- --wgpu`
//!
//! Keys: Up/Down or j/k to move, Enter to select, Esc or q to quit.

use raclettui::{
    WindowBuilder,
    events::{AxisCode, KeyCode, MouseEvent, WindowEvent, WindowEventQueue},
    layer::{KeyboardInteractivity, Layer},
};
use ratatui::{
    Terminal,
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListState, Paragraph},
};

const ENTRIES: [&str; 6] = ["Terminal", "Browser", "Files", "Editor", "Music", "Settings"];

fn main() -> Result<(), raclettui::Error> {
    // no anchors: the compositor centers the surface
    let builder = WindowBuilder::new()
        .set_namespace("menu")
        .set_width(360)
        .set_height(240)
        .set_layer(Layer::Overlay)
        .set_keyboard_interactivity(KeyboardInteractivity::Exclusive)
        .set_bg_alpha(0.85);

    // both backends implement ratatui's Backend trait with the same error type
    let selected = if std::env::args().any(|arg| arg == "--wgpu") {
        let window = builder.init_wgpu()?;
        let events = window.get_event_queue();
        run(Terminal::new(window)?, events)?
    } else {
        let window = builder.init_cpu()?;
        let events = window.get_event_queue();
        run(Terminal::new(window)?, events)?
    };

    if let Some(entry) = selected {
        println!("{entry}");
    }
    Ok(())
}

fn run<B>(mut terminal: Terminal<B>, events: WindowEventQueue) -> Result<Option<&'static str>, raclettui::Error>
where
    B: Backend<Error = raclettui::Error>,
{
    let mut state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|f| {
            let [list_area, help_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

            let list = List::new(ENTRIES)
                .block(Block::bordered().title(" Launch "))
                .style(Style::new().fg(Color::White))
                // REVERSED swaps the foreground and background of the selected row
                .highlight_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::REVERSED | Modifier::BOLD))
                .highlight_symbol("> ");
            f.render_stateful_widget(list, list_area, &mut state);

            let help = Paragraph::new("↑/↓ move · enter select · esc quit")
                .style(Style::new().fg(Color::DarkGray));
            f.render_widget(help, help_area);
        })?;

        for event in events.drain() {
            match event {
                WindowEvent::Keyboard(key) if key.is_press() || key.is_repeat() => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => state.select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => state.select_next(),
                    KeyCode::Enter => return Ok(state.selected().map(|i| ENTRIES[i.min(ENTRIES.len() - 1)])),
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                    _ => {}
                },
                WindowEvent::Pointer(MouseEvent::Axis { code: AxisCode::VerticalScroll, value }) => {
                    if value > 0.0 {
                        state.select_next();
                    } else {
                        state.select_previous();
                    }
                }
                WindowEvent::Closed => return Ok(None),
                _ => {}
            }
        }
    }
}
