# 🚀 TODOs

- [x] Force a font loading
- [ ] Grapheme cluster handling for backend
- [ ] Ratatui backend handle cell data better with modifiers, skip, underline ...
  - [x] reversed, hidden, skip (both backends), bold and italic (wgpu)
  - [ ] underline, crossed out, dim, bold and italic for the cpu backend
- [ ] Vulkan backend libraries in nix
- [ ] Optimise background rendering using instances
- [x] Resize events handling for wgpu
- [ ] Popup window widget
- [ ] Client side key repeat using `wl_keyboard.repeat_info`
- [ ] HiDPI / fractional scale support
- [ ] Draw a cursor at the position set by `Frame::set_cursor_position`
- [ ] Double buffering for the cpu backend using `wl_buffer.release`
