use glyphon::FontSystem;
use wgpu::util::DeviceExt;

use crate::colors::{self, RaclettuiColor};
use crate::builder::WindowBuilder;

const CELL_WIDTH_F: f32 = 1.4;
const CELL_HEIGHT_F: f32 = 1.1;

pub struct TerminalGrid {
    pub physical_width: f32,
    pub physical_height: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub rows: u32,
    pub cols: u32,
    pub ch_buffer: Vec<glyphon::Buffer>,
    pub bg_buffer: Vec<RaclettuiColor>,

    pub fg_alpha: f32,
    pub bg_alpha: f32,
}
impl TerminalGrid {
    fn new(
        physical_width: f32,
        physical_height: f32,
        cell_width: f32,
        cell_height: f32,
        font_system: &mut FontSystem,
        font_size: f32,
        line_height: f32,
        fg_alpha: f32,
        bg_alpha: f32,
    ) -> Self {
        let cols = (physical_width / cell_width) as u32;
        let rows = (physical_height / cell_height) as u32;

        Self {
            physical_width,
            physical_height,
            cell_width,
            cell_height,
            rows,
            cols,
            ch_buffer: vec![
                glyphon::Buffer::new(font_system, glyphon::Metrics::new(font_size, line_height));
                (rows*cols) as usize
            ],
            bg_buffer: vec![RaclettuiColor::new(); (rows* cols) as usize],
            fg_alpha,
            bg_alpha,
        }
    }

    // to add italics bolds ... needs a way to force a font.
    pub fn set_ch(
        &mut self,
        row: u32,
        col: u32,
        ch: char,
        color: RaclettuiColor,
        font_system: &mut FontSystem,
    )
    {
        if row >= self.rows || col >= self.cols {
            return;
        }

        let idx = (row * self.cols + col) as usize;
        let alpha = (self.fg_alpha * 255.) as u8;
        self.ch_buffer[idx].set_text(
            font_system,
            ch.to_string().as_str(),
            &glyphon::Attrs::new()
                .color(color.into())
                .family(glyphon::cosmic_text::Family::Monospace),
                // .style(cosmic_text::Style::Italic)
            glyphon::Shaping::Advanced,
            None,
        );
        self.ch_buffer[idx].shape_until_scroll(font_system, false);
    }

    pub fn set_bg(
        &mut self,
        row: u32,
        col: u32,
        color: RaclettuiColor,
    )
    {
        if row >= self.rows || col >= self.cols {
            panic!("src/wgpu/render.rs setting values out of bounds")
        }
        let idx = (row * self.cols + col) as usize;
        self.bg_buffer[idx] = color;
    }

    fn get_text_areas(&self) -> Vec<glyphon::TextArea<'_>> {
        let mut text_areas = Vec::new();

        for i in 0..self.rows {
            for j in 0..self.cols {
                let idx = (i * self.cols + j) as usize;
                let buffer = &self.ch_buffer[idx];

                let text_area = glyphon::TextArea {
                    buffer,
                    left: (j as f32 * self.cell_width),
                    top: (i as f32 * self.cell_height),
                    scale: 1.0,
                    bounds: glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: self.physical_width as i32,
                        bottom: self.physical_height as i32,
                    },
                    default_color: glyphon::Color::rgba(255, 255, 255, 255),
                    custom_glyphs: &[]

                };
                text_areas.push(text_area);

            }
        }
        text_areas

    }

}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct GridRenderer {
    pub grid: TerminalGrid,
    pub font_system: FontSystem,
    // font_size: f32,

    window_width: u32,
    window_height: u32,
    cell_width: f32,
    cell_height: f32,
    quad_pipeline: wgpu::RenderPipeline,

    swash_cache: glyphon::SwashCache,
    atlas: glyphon::TextAtlas,
    viewport: glyphon::Viewport,
    text_renderer: glyphon::TextRenderer,

}

impl GridRenderer {
    pub fn new(
        mut font_system: FontSystem,
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_builder: &WindowBuilder,

    ) -> Self {

        // text renderer and grid initialisation.
        let swapchain_format = surface_config.format;
        let swash_cache = glyphon::SwashCache::new();

        let cache = glyphon::Cache::new(device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas = glyphon::TextAtlas::new(&device, queue, &cache, swapchain_format);
        let text_renderer = glyphon::TextRenderer::new(
                &mut atlas,
                device,
                wgpu::MultisampleState::default(),
                None
            );

        // line height same as font size, can add a multiplier to it for breating room.
        let font_size = window_builder.font_size;
        // increasing the line height by a factor. // to finetune
        let line_height =  CELL_HEIGHT_F * font_size;
        let mut text_buffer = glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(font_size, line_height));

        // calculating cell height and width
        text_buffer.set_text(
            &mut font_system,
            "M",
            &glyphon::Attrs::new().family(glyphon::Family::Monospace),
            glyphon::Shaping::Advanced,
            None
         );
        let mut layout = text_buffer.layout_runs();
        let char_width = layout.next().unwrap().line_w;

        let physical_width = surface_config.width as f32;
        let physical_height = surface_config.height as f32;

        let cell_height = line_height;
        // adding some padding because characters overflow.
        let cell_width = char_width * CELL_WIDTH_F;

        let grid = TerminalGrid::new(
            physical_width,
            physical_height,
            cell_width,
            cell_height,
            &mut font_system,
            font_size,
            line_height,
            window_builder.fg_alpha,
            window_builder.bg_alpha,
        );

        // starting to initialise quad pipeline
        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into())
        });

        let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let quad_vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                }
            ]
        };

        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Render Pipeline"),
            layout: Some(&quad_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &quad_shader,
                entry_point: Some("quad_vs_main"),
                buffers: &[quad_vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &quad_shader,
                entry_point: Some("quad_fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(
                        wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }
                    ),
                    write_mask: wgpu::ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });


        Self {
            font_system,
            grid,
            window_width: surface_config.width,
            window_height: surface_config.height,
            cell_width,
            cell_height,

            swash_cache,
            text_renderer,
            viewport,
            atlas,

            quad_pipeline,
        }
    }

    // to be optimised to use instancing
    pub fn render_background(
        &self,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass,
    ) {
        // multiply by 6 because of triangles stuffs
        let mut vertices = Vec::with_capacity((self.grid.cols * self.grid.rows * 6) as usize);
        for row in 0..self.grid.rows {
            for col in 0..self.grid.cols {

                let idx = (row * self.grid.cols + col) as usize;
                let bg_color = self.grid.bg_buffer[idx];
                let x = col as f32 * self.cell_width as f32;
                let y = row as f32 * self.cell_height as f32;

                let x0 = (x / self.window_width as f32) * 2.0 - 1.0;
                let y0 = 1.0 - (y / self.window_height as f32) * 2.0;
                let x1 = ((x + self.cell_width as f32) / self.window_width as f32) * 2.0 - 1.0;
                let y1 = 1.0 - ((y + self.cell_height as f32) / self.window_height as f32) * 2.0;
                // let color = colors::linear_color(bg_color, self.grid.bg_alpha);
                let color = bg_color.to_linear();

                vertices.extend_from_slice(&[
                    QuadVertex { position: [x0, y0], color },
                    QuadVertex { position: [x1, y0], color },
                    QuadVertex { position: [x0, y1], color },
                    QuadVertex { position: [x1, y0], color },
                    QuadVertex { position: [x1, y1], color },
                    QuadVertex { position: [x0, y1], color },
                ]);
            }
        }

        if vertices.is_empty() {
            return ;
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Grid Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,

            },
        );

        render_pass.set_pipeline(&self.quad_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    pub fn render_text(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass,
    ) {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: self.window_width,
                height: self.window_height,
            },
        );

        let text_areas = self.grid.get_text_areas();

        self.text_renderer
            .prepare(
                &device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
                .unwrap();
        self.text_renderer.render(&self.atlas, &self.viewport, render_pass).unwrap();

    }

    pub fn cell_width(&self) -> f32 {
        self.cell_width
    }
    pub fn cell_height(&self) -> f32 {
        self.cell_height
    }
}
