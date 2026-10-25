<p align="center">
<a href="https://github.com/ishrut/raclettui"><img src="https://raw.githubusercontent.com/ishrut/raclettui/main/assets/raclettuilogo.png" width="500"></a>
</p>

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/raclettui.svg)](https://crates.io/crates/raclettui)
[![Docs.rs](https://docs.rs/raclettui/badge.svg)](https://docs.rs/raclettui)
[![License](https://img.shields.io/crates/l/raclettui.svg)](#license)

</div>

# Raclettui

Build terminal-themed Wayland layer shell applications with Rust. Powered by [Ratatui].

## About

**Raclettui** creates windows using the [wlr layer shell] protocol and implements the [Ratatui] `Backend` trait for them, so terminal style UIs can be drawn into bars, panels, launchers, notifications, overlays and lock screens. Any Ratatui application can be turned into a layer shell window by switching its backend.

It has two rendering backends:

| Backend | Created with | Rendering |
|---------|--------------|-----------|
| `CpuWindow` | `WindowBuilder::init_cpu()` | Software rendering into a shared memory buffer. No GPU needed. |
| `WgpuWindow` | `WindowBuilder::init_wgpu()` | Hardware accelerated with `wgpu` and `glyphon`. Supports bold and italic text. |

This project is still under active development. Expect bugs and breaking changes, but it works if you would like to try it out. It was developed to cook a UI for my own project, as no UI library in the Rust ecosystem implementing the layer shell protocol was to my liking.

## Quickstart

Add **Raclettui** and **Ratatui** as dependencies:

```sh
cargo add raclettui ratatui
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
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

fn main() -> Result<(), raclettui::Error> {
    let window = WindowBuilder::new()
        .set_namespace("example")
        .set_width(300)
        .set_height(300)
        .set_layer(Layer::Top)
        .set_anchor(Anchor::Top)
        .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
        // .set_font_path("fonts/Some-Mono-Font.ttf") // defaults to the system monospace font
        .set_font_size(18.)
        .set_bg_alpha(0.5)
        .init_wgpu()?; // or .init_cpu()?

    // grab the event queue before the terminal takes ownership of the window
    let events = window.get_event_queue();
    let mut terminal = Terminal::new(window)?;

    loop {
        // blocks until the compositor is ready for the next frame
        terminal.draw(|f| {
            let paragraph = Paragraph::new("Hello World")
                .block(Block::bordered())
                .style(Style::new().fg(Color::Red).bg(Color::Blue));
            f.render_widget(paragraph, f.area());
        })?;

        for event in events.drain() {
            match event {
                // key events are sent for both presses and releases
                WindowEvent::Keyboard(key) if key.is_press() && key.code == KeyCode::Char('q') => return Ok(()),
                WindowEvent::Closed => return Ok(()),
                _ => {}
            }
        }
    }
}
```

## Usage

### Window configuration

`WindowBuilder` exposes the layer shell options:

| Method | Description |
|--------|-------------|
| `set_width` / `set_height` | Size in pixels. `0` lets the compositor decide, which requires anchoring to both opposite edges. |
| `set_anchor` / `set_anchors` | Screen edges to attach to. Anchors combine: `Top + Left + Right` makes a full-width bar. |
| `set_layer` | `Background`, `Bottom`, `Top` or `Overlay`. |
| `set_margin` | Distance from the anchored edges, `(top, right, bottom, left)`. |
| `set_exclusive_zone` | Space reserved for the surface so other windows do not cover it. |
| `set_exclusive_edge` | Edge the exclusive zone applies to (layer shell v5). |
| `set_keyboard_interactivity` | `None`, `Exclusive` or `OnDemand`. Needed to receive keyboard events. |
| `set_namespace` | Name compositors use to identify the surface, e.g. in window rules. |
| `set_font_path` / `set_font_size` | Monospace font file and size in pixels. Defaults to the system monospace font at 17px. |
| `set_bg_alpha` / `set_fg_alpha` | Opacity of the background and of the text, from `0.0` to `1.0`. |

### Events

Input is read from Wayland while the window waits for the compositor, which happens inside every `terminal.draw`. Events are pushed to a `WindowEventQueue` obtained with `get_event_queue()`:

- `WindowEvent::Keyboard(KeyEvent)` with the key code, `is_press()` / `is_release()` / `is_repeat()` and the shift, ctrl and alt modifiers
- `WindowEvent::Pointer(MouseEvent)` for motion, enter, leave, buttons and scrolling, in surface pixels
- `WindowEvent::Resize { width, height }` when the compositor resizes the surface, the Ratatui grid follows automatically
- `WindowEvent::Closed` when the compositor closes the surface

## Examples

```sh
cargo run --example cpu_ratatui      # minimal app on the CPU backend
cargo run --example wgpu_ratatui     # the same app on the wgpu backend
cargo run --example menu             # keyboard / mouse driven launcher menu (add -- --wgpu for the GPU)
cargo run --example bar              # full-width status bar with an exclusive zone
cargo run --example wgpu_grid        # draws cells directly, without Ratatui
```

Set `RACLETTUI_FONT=/path/to/font.ttf` to try the `cpu_ratatui` and `wgpu_ratatui` examples with a specific font.

## Demo

Converting the [bluetui] app interface into a layer shell window is as simple as switching the backend.
However, converting the crossterm events to window events is still a work in progress.

<a><img src="https://raw.githubusercontent.com/ishrut/raclettui/main/assets/bluetuidemo.png" width="500"></a>

## Requirements

- A Wayland compositor implementing `wlr-layer-shell`, such as sway, Hyprland, niri, river, KDE Plasma, Wayfire or labwc. GNOME does not support it.
- `libwayland-client` and `libxkbcommon`.
- For the wgpu backend, a Vulkan or OpenGL ES capable driver.

A Nix development shell with all dependencies is provided: `nix develop`.

## Limitations

- Only the first character of each cell is drawn, grapheme clusters are not supported yet.
- Underlined and crossed out text are not rendered. Bold and italic are only supported by the wgpu backend, reversed and hidden by both.
- No cursor is drawn and key repeat is not emulated client side.
- HiDPI / fractional scaling is not handled yet.

## Documentation

- [API Documentation](https://docs.rs/raclettui)
- [Ratatui documentation](https://ratatui.rs)
- [wlr layer shell protocol][wlr layer shell]

## Acknowledgements

Thanks to [Ratzilla] projects for the inspiration.
Thanks to [Ratatui] for providing the core UI components.

[Ratatui]: https://ratatui.rs
[Ratzilla]: https://github.com/ratatui/ratzilla
[bluetui]: https://github.com/pythops/bluetui
[wlr layer shell]: https://wayland.app/protocols/wlr-layer-shell-unstable-v1

## Contributing

Pull requests are welcome!
Please see the todo list: [TODO](./TODO.md)

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

## Copyright

Copyright © 2025, Ishrut
