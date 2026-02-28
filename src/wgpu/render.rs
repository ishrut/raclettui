use wgpu::util::DeviceExt;
use std::collections::HashMap;

use super::window::Grid;
use crate::colors;
use crate::builder::WindowBuilder;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[derive(Clone)]
struct GlyphTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

pub struct GridRenderer {
    pub grid: Grid,
    font: fontdue::Font,
    font_size: f32,

    window_width: u32,
    window_height: u32,
    cell_width: u32,
    cell_height: u32,
    quad_pipeline: wgpu::RenderPipeline,

    glyph_cache: HashMap<char, GlyphTexture>,
    text_pipeline: wgpu::RenderPipeline,
    text_bind_group_layout: wgpu::BindGroupLayout,
    text_sampler: wgpu::Sampler,

    bg_alpha: f32,
    fg_alpha: f32,

}

impl GridRenderer {
    pub fn new(
        font: fontdue::Font,
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        window_builder: &WindowBuilder,

    ) -> Self {

        // initialises grid to track states
        let m_metrics = font.metrics('M', window_builder.font_size);
        let line_metrics = font.horizontal_line_metrics(window_builder.font_size).unwrap();
        let cell_height = (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap).ceil() as u32;
        let cell_width = m_metrics.width as u32;
        let rows = surface_config.height / cell_height;
        let cols = surface_config.width / cell_width;

        let grid = Grid::new(rows as usize, cols as usize);

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
                            alpha: wgpu::BlendComponent::REPLACE,
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

        // text pipeline initialisation
        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });
        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering
                        ),
                        count: None,
                    },
                ],
            });

        let text_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Text Pipeline Layout"),
                bind_group_layouts: &[&text_bind_group_layout],
                immediate_size: 0,
            });

        let text_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as u64,
                    shader_location: 2,
                },
            ],
        };

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: Some("text_vs_main"),
                buffers: &[text_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: Some("text_fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            font,
            font_size: window_builder.font_size,
            grid,
            window_width: surface_config.width,
            window_height: surface_config.height,
            cell_width,
            cell_height,

            quad_pipeline,

            text_pipeline,
            text_bind_group_layout,
            text_sampler,

            glyph_cache: HashMap::<char, GlyphTexture>::new(),

            fg_alpha: window_builder.fg_alpha,
            bg_alpha: window_builder.bg_alpha,
        }
    }

    // to be optimised to use instancing
    pub fn render_background(
        &self,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass,
    ) {
        // multiply by 6 because of triangles stuffs
        let mut vertices = Vec::with_capacity(self.grid.cols() * self.grid.rows() * 6);
        for row in 0..self.grid.rows() {
            for col in 0..self.grid.cols() {
                let cell = self.grid.get_cell(row, col).unwrap();
                let x = col as f32 * self.cell_width as f32;
                let y = row as f32 * self.cell_height as f32;

                let x0 = (x / self.window_width as f32) * 2.0 - 1.0;
                let y0 = 1.0 - (y / self.window_height as f32) * 2.0;
                let x1 = ((x + self.cell_width as f32) / self.window_width as f32) * 2.0 - 1.0;
                let y1 = 1.0 - ((y + self.cell_height as f32) / self.window_height as f32) * 2.0;
                let color = colors::linear_color(cell.bg_color, self.bg_alpha);

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

    pub fn cell_width(&self) -> u32 {
        self.cell_width
    }
    pub fn cell_height(&self) -> u32 {
        self.cell_height
    }

    fn get_or_create_glyph(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ch: char,
    ) -> &GlyphTexture {
        if !self.glyph_cache.contains_key(&ch) {
            let (metrics, bitmap) = self.font.rasterize(ch, self.font_size);

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Glyph Texture"),
                size: wgpu::Extent3d {
                    width: metrics.width as u32,
                    height: metrics.height as u32,
                    depth_or_array_layers: 1,
                },
                format: wgpu::TextureFormat::R8Unorm,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                texture.as_image_copy(),
                 &bitmap,
                 wgpu::TexelCopyBufferLayout {
                     offset: 0,
                      bytes_per_row: Some(metrics.width as u32),
                       rows_per_image: None
                   },
                wgpu::Extent3d {
                    width: metrics.width as u32,
                    height: metrics.height as u32,
                    depth_or_array_layers: 1,
                },
            );

            let view = texture.create_view(&Default::default());

            self.glyph_cache.insert(
                ch,
                GlyphTexture {
                    texture,
                    view,
                    width: metrics.width as u32,
                    height: metrics.height as u32,
                },
            );
        }

        self.glyph_cache.get(&ch).unwrap()
    }

    pub fn render_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
    ) {
        render_pass.set_pipeline(&self.text_pipeline);

        for row in 0..self.grid.rows() {
            for col in 0..self.grid.cols() {
                let cell = self.grid.get_cell(row, col).unwrap().clone();

                if cell.ch == ' ' {
                    continue;
                }

                let glyph = self.get_or_create_glyph(device, queue, cell.ch).clone();

                let x = col as f32 * self.cell_width as f32;
                let y = row as f32 * self.cell_height as f32;

                let glyph_width = glyph.width as f32;
                let glyph_height = glyph.height as f32;

                // Optional: center horizontally in cell
                let x_offset = (self.cell_width as f32 - glyph_width) * 0.5;

                // Baseline alignment (important!)
                let y_offset = self.cell_height as f32 - glyph_height;

                let gx = x + x_offset;
                let gy = y + y_offset;

                let x0 = (gx / self.window_width as f32) * 2.0 - 1.0;
                let y0 = 1.0 - (gy / self.window_height as f32) * 2.0;
                let x1 = ((gx + glyph_width) / self.window_width as f32) * 2.0 - 1.0;
                let y1 = 1.0 - ((gy + glyph_height) / self.window_height as f32) * 2.0;

                let color = colors::linear_color(cell.fg_color, self.fg_alpha);

                let vertices = [
                    TextVertex { position: [x0, y0], tex_coords: [0.0, 0.0], color },
                    TextVertex { position: [x1, y0], tex_coords: [1.0, 0.0], color },
                    TextVertex { position: [x0, y1], tex_coords: [0.0, 1.0], color },
                    TextVertex { position: [x1, y0], tex_coords: [1.0, 0.0], color },
                    TextVertex { position: [x1, y1], tex_coords: [1.0, 1.0], color },
                    TextVertex { position: [x0, y1], tex_coords: [0.0, 1.0], color },
                ];

                let vertex_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Text Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.text_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&glyph.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.text_sampler),
                        },
                    ],
                    label: Some("Text Bind Group"),
                });

                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..6, 0..1);
            }
        }
    }
}
