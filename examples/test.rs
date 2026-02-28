use raclettui::{WindowBuilder, Layer, Anchor};

fn main() {
    let mut window = WindowBuilder::new()
        .set_namespace("example")
        .set_width(300)
        .set_height(300)
        .set_layer(Layer::Top)
        .set_anchor(Anchor::Top)
        .set_keyboard_interactivity(raclettui::KeyboardInteractivity::OnDemand)
        .set_font_size(18.)
        .bg_alpha(0.5)
        .init_wgpu();

    println!("window width: {}, window height: {}", window.width(), window.height());
    println!("cell width: {}, cell height: {}", window.grid_renderer.cell_width(), window.grid_renderer.cell_height());
    println!("rows: {}, cols: {}", window.grid_renderer.grid.rows(), window.grid_renderer.grid.cols());

    window.grid_renderer.grid.set_cell(
        0,
         0,
         'M',
         (0, 255, 0),
         (255, 0, 0),
      );

    window.grid_renderer.grid.set_cell(
        5,
         5,
         'p',
         (0, 255, 0),
         (0, 0, 0),
      );

    window.grid_renderer.grid.set_cell(
        window.grid_renderer.grid.rows() -1,
         window.grid_renderer.grid.cols() -1 ,
         '|',
         (255, 0, 0),
         (0, 0, 0),
      );
    loop {
        window.update();
        if window.is_redraw() {
            window.render();
        }
    }

}
