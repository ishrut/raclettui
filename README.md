<p align="center">
<a href="https://github.com/ishrut/raclettui"><img src="https://github.com/ishrut/raclettui/blob/main/assets/raclettuilogo.png" width="500"></a>
</p>

<div align="center">

[![Repo]()]()
[![Crate]()]()
[![Docs]()]()

</div>

# Raclettui

Build terminal-themed wayland layer shell applications with Rust. Powered by [Ratatui].

## About
**Raclettui** allows you to create windows using wayland layer shell protocol and implements the [Ratatui] Backend trait to draw terminal style UIs.
Layer shell protocol is usually used to create menus, bars and lockscreens.
This project is still under active development. Expect bugs and breaking changes. However, it works if you would like to try it out.
It has cpu and wgpu rendering backends. This library was developped to cook a UI for my project as no UI libraries in the rust ecosystem implementing the layer shell protocol were to my liking.

## Quickstart

### Setup

Add **Raclettui** as a dependency in your `Cargo.toml`:

```sh
cargo add raclettui
```

Here is a minimal example:

```rust no_run

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
            .set_font_path("fonts/Some-Mono-Font.ttf")
            .set_font_size(18.)
            .bg_alpha(0.5)
            .init_wgpu();

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
```

## Demo

## Documentation

- [API Documentation]()
- [Backends]()
- [Widgets]()

## Acknowledgements

Thanks to [Ratzilla] projects for the inspiration.
Thanks to [Ratatui] for providing the core UI components.

[Ratatui]: https://ratatui.rs
[Ratzilla]: https://github.com/ratatui/ratzilla

## Contributing

Pull requests are welcome!
Please see todo list: [TODO](./TODO.md)

## Copyright

Copyright © 2025, [Ishrut](mailto:)
