<p align="center">
<a href="https://github.com/ishrut/raclettui"><img src="https://github.com/ishrut/raclettui/blob/main/assets/raclettuilogo.png" width="500"></a>
</p>

<div align="center">

</div>

# Raclettui

Build terminal-themed wayland layer shell applications with Rust. Powered by [Ratatui].

## About
**Raclettui** allows you to create windows using wayland layer shell protocol and implements the [Ratatui] Backend trait to draw terminal style UIs.
Layer shell protocol is usually used to create menus, bars and lockscreens.
This project is still under active development. Expect bugs and breaking changes. However, it works if you would like to try it out.
It uses [beamterm-core] crate as terminal renderer. This library was developped to cook a UI for my project as no UI libraries in the rust ecosystem implementing the layer shell protocol were to my liking.

## Quickstart

### Setup

Add **Raclettui** as a dependency in your `Cargo.toml`:

```sh
cargo add raclettui
```

Checkout examples/hello.rs for a minimal example.

## Demo
Converting the bluetui app interface in to a layer shell window is as simple as switching the backend.
However, converting the crossterm events to window events is still a work in progress.
<a><img src="https://github.com/ishrut/raclettui/blob/main/assets/bluetuidemo.png" width="500"></a>


## Documentation

- [API Documentation]()
- [Backends]()
- [Widgets]()

## Acknowledgements

Thanks to [Ratzilla] projects for the inspiration.
Thanks to [Ratatui] for providing the core UI components.

[Ratatui]: https://ratatui.rs
[Ratzilla]: https://github.com/ratatui/ratzilla
[beamterm-core]: https://github.com/junkdog/beamterm

## Contributing

Pull requests are welcome!
Please see todo list: [TODO](./TODO.md)

## Copyright

Copyright © 2025, [Ishrut](mailto:)
