use anyhow::anyhow;
use crevice::std140::AsStd140;
use hawthorn::math::Quat;
use image::GenericImageView;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::app::{
    Camera, DrawCommand, EntityStd140, GlobalsStd140, Mesh, MeshBuffers, SkyboxStd140, SurfaceInfo,
    UniformsBuffer, World,
};

type SystemFn = dyn FnMut(&mut World);

pub struct App {
    pub world: World,

    name: String,
    meshes_to_create: HashMap<String, Mesh>,
    systems: Vec<Box<SystemFn>>,

    last_frame_start_time: Option<Instant>,
    wgpu_context: Option<WgpuContext>,
}

struct WgpuContext {
    window: Arc<Window>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,

    device: wgpu::Device,
    queue: wgpu::Queue,

    skybox_texture_bind_group: wgpu::BindGroup,
    shadow_map_bind_group: wgpu::BindGroup,

    shadow_map_texture_view: wgpu::TextureView,
    depth_texture_view: wgpu::TextureView,

    skybox_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    entity_pipeline: wgpu::RenderPipeline,

    skybox_uniforms: UniformsBuffer,
    global_uniforms: UniformsBuffer,
    entity_uniforms: UniformsBuffer,

    builtin_meshes: BuiltinMeshes,
    mesh_buffers: HashMap<String, MeshBuffers>,
}

struct BuiltinMeshes {
    cube: MeshBuffers,
}

const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SHADOW_MAP_SIZE: u32 = 2_048;

impl App {
    pub fn new(name: impl Into<String>) -> Self {
        App {
            name: name.into(),
            world: World::default(),
            meshes_to_create: HashMap::new(),
            systems: Vec::new(),

            last_frame_start_time: None,
            wgpu_context: None,
        }
    }

    pub fn create_mesh(&mut self, name: &str, mesh: Mesh) {
        self.meshes_to_create.insert(String::from(name), mesh);
    }

    pub fn add_system<T>(&mut self, system: T)
    where
        T: FnMut(&mut World) + 'static,
    {
        self.systems.push(Box::new(system));
    }

    pub fn start(mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::builder().build()?;

        event_loop.run_app(&mut self)?;

        Ok(())
    }

    fn handle_window_resize(&mut self, width: u32, height: u32) {
        let Some(wgpu_context) = &mut self.wgpu_context else {
            return;
        };

        let width = width.max(1);
        let height = height.max(1);

        self.world.surface_info = SurfaceInfo { width, height };

        wgpu_context.surface_config.width = width;
        wgpu_context.surface_config.height = height;

        wgpu_context.depth_texture_view =
            new_depth_texture_view(&wgpu_context.device, (width, height));

        wgpu_context
            .surface
            .configure(&wgpu_context.device, &wgpu_context.surface_config);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.wgpu_context.is_none() {
            let mut wgpu_context = pollster::block_on(WgpuContext::try_new(&self.name, event_loop))
                .unwrap_or_else(|e| panic!("failed to create wgpu context: {e:?}"));

            for (handle, mesh) in self.meshes_to_create.drain() {
                let buffers = MeshBuffers::from_mesh(mesh, &wgpu_context.device);

                wgpu_context.mesh_buffers.insert(handle, buffers);
            }

            self.wgpu_context = Some(wgpu_context);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(wgpu_context) = self.wgpu_context.as_ref() {
            wgpu_context.window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(wgpu_context) = &mut self.wgpu_context else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                let now = Instant::now();

                let last_frame_start_time = match self.last_frame_start_time {
                    Some(t) => t,
                    None => now,
                };

                self.last_frame_start_time = Some(now);
                self.world.frame_duration = now.saturating_duration_since(last_frame_start_time);

                for system in &mut self.systems {
                    (system)(&mut self.world);
                }

                match wgpu_context
                    .render_frame(&self.world.camera, self.world.draw_commands.drain(..))
                {
                    Ok(()) => {}
                    Err(e) => {
                        log::error!("frame render failed: {e:?}");
                    }
                };

                self.world.input_state.end_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.world.input_state.handle_key_event(&event);

                // Exit the app when Escape is pressed
                if event.logical_key == Key::Named(NamedKey::Escape)
                    && event.state == ElementState::Pressed
                {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                self.handle_window_resize(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        };
    }
}

impl WgpuContext {
    async fn try_new(name: &str, event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        let window_attribs = WindowAttributes::default()
            .with_title(name)
            .with_inner_size(LogicalSize::new(1200, 800))
            .with_resizable(true);

        let window = event_loop.create_window(window_attribs).map(Arc::new)?;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        log::info!("{:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device descriptor"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_size = window.inner_size();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        let surface_present_mode = surface_capabilities
            .present_modes
            .iter()
            .find(|&&p| p == wgpu::PresentMode::AutoVsync)
            .copied()
            .unwrap_or(surface_capabilities.present_modes[0]);
        let surface_alpha_mode = surface_capabilities
            .alpha_modes
            .iter()
            .find(|&&p| p == wgpu::CompositeAlphaMode::Opaque)
            .copied()
            .unwrap_or(surface_capabilities.alpha_modes[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_size.width,
            height: surface_size.height,
            present_mode: surface_present_mode,
            alpha_mode: surface_alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        };

        let skybox_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/skybox.wgsl"));
        let entity_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/entity.wgsl"));

        let skybox_uniforms_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("skybox uniforms bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let skybox_uniforms = UniformsBuffer::new(
            &device,
            &skybox_uniforms_layout,
            SkyboxStd140::VERTEX_BINDING_SIZE,
            SkyboxStd140::FRAGMENT_BINDING_SIZE,
        );

        let global_uniforms_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("global uniforms bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let global_uniforms = UniformsBuffer::new(
            &device,
            &global_uniforms_layout,
            GlobalsStd140::VERTEX_BINDING_SIZE,
            GlobalsStd140::FRAGMENT_BINDING_SIZE,
        );

        let entity_uniforms_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("entity uniforms bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let entity_uniforms = UniformsBuffer::new(
            &device,
            &entity_uniforms_layout,
            EntityStd140::VERTEX_BINDING_SIZE,
            EntityStd140::FRAGMENT_BINDING_SIZE,
        );

        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let skybox_texture_view = new_cubemap_view(
            &device,
            &queue,
            &[
                "res/skybox/px.png",
                "res/skybox/nx.png",
                "res/skybox/py.png",
                "res/skybox/ny.png",
                "res/skybox/pz.png",
                "res/skybox/nz.png",
            ],
        )?;

        let skybox_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("skybox texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let skybox_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skybox texture bind group"),
            layout: &skybox_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&skybox_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler_linear),
                },
            ],
        });

        let skybox_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("skybox render pipeline layout"),
                bind_group_layouts: &[
                    Some(&skybox_uniforms_layout),
                    Some(&skybox_texture_bind_group_layout),
                ],
                ..Default::default()
            });

        let skybox_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("skybox render pipeline"),
                layout: Some(&skybox_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &skybox_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(MeshBuffers::positions_only_vertex_layout())],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &skybox_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: Default::default(),
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_map_texture_view =
            new_depth_texture_view(&device, (SHADOW_MAP_SIZE, SHADOW_MAP_SIZE));

        let shadow_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow render pipeline layout"),
                bind_group_layouts: &[Some(&global_uniforms_layout), Some(&entity_uniforms_layout)],
                ..Default::default()
            });

        let shadow_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shadow render pipeline"),
                layout: Some(&shadow_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &entity_shader,
                    entry_point: Some("vs_shadow"),
                    buffers: &[Some(MeshBuffers::positions_only_vertex_layout())],
                    compilation_options: Default::default(),
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_TEXTURE_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        let depth_texture_view =
            new_depth_texture_view(&device, (surface_config.width, surface_config.height));

        let shadow_map_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow map bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        let shadow_map_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow map bind group"),
            layout: &shadow_map_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&depth_sampler),
                },
            ],
        });

        let entity_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("entity render pipeline layout"),
                bind_group_layouts: &[
                    Some(&global_uniforms_layout),
                    Some(&entity_uniforms_layout),
                    Some(&shadow_map_bind_group_layout),
                ],
                ..Default::default()
            });

        let entity_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("entity render pipeline"),
                layout: Some(&entity_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &entity_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(MeshBuffers::standard_vertex_layout())],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &entity_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth_texture_view.texture().format(),
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        surface.configure(&device, &surface_config);

        Ok(Self {
            window,
            surface_config,
            surface,
            queue,
            shadow_map_texture_view,
            depth_texture_view,
            skybox_texture_bind_group,
            shadow_map_bind_group,
            skybox_pipeline: skybox_render_pipeline,
            shadow_pipeline: shadow_render_pipeline,
            entity_pipeline: entity_render_pipeline,
            skybox_uniforms,
            global_uniforms,
            entity_uniforms,
            builtin_meshes: BuiltinMeshes::new(&device),
            mesh_buffers: HashMap::new(),
            device,
        })
    }

    fn surface_info(&self) -> SurfaceInfo {
        SurfaceInfo {
            width: self.surface_config.width,
            height: self.surface_config.height,
        }
    }

    fn render_frame(
        &self,
        camera: &Camera,
        draw_commands: impl IntoIterator<Item = DrawCommand>,
    ) -> anyhow::Result<()> {
        let draw_commands = draw_commands.into_iter().collect::<Vec<_>>();

        let entities_std140 = draw_commands
            .iter()
            .cloned()
            .map(EntityStd140::from_draw_command)
            .map(|data| data.as_std140())
            .collect::<Vec<_>>();

        self.skybox_uniforms.update(
            &self.device,
            &self.queue,
            &[
                SkyboxStd140::new(camera, &self.surface_info(), Quat::IDENTITY)
                    .as_std140()
                    .as_bytes(),
            ],
        );
        self.global_uniforms.update(
            &self.device,
            &self.queue,
            &[GlobalsStd140::new(camera, &self.surface_info())
                .as_std140()
                .as_bytes()],
        );
        self.entity_uniforms.update(
            &self.device,
            &self.queue,
            &entities_std140
                .iter()
                .map(|data| data.as_bytes())
                .collect::<Vec<_>>(),
        );

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(_)
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);

                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(anyhow!("a validation error occurred"));
            }
        };

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_scene encoder"),
                });

        {
            let mut shadow_render_pass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("shadow render pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.shadow_map_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

            shadow_render_pass.set_pipeline(&self.shadow_pipeline);

            shadow_render_pass.set_bind_group(0, &self.global_uniforms.bind_group, &[]);

            for (i, draw_command) in draw_commands.iter().enumerate() {
                let Some(mesh_buffers) = self.mesh_buffers.get(&draw_command.mesh_name) else {
                    log::warn!("no mesh exists with name {:?}", draw_command.mesh_name);
                    continue;
                };

                shadow_render_pass.set_vertex_buffer(0, mesh_buffers.vertex_buffer.slice(..));
                shadow_render_pass.set_index_buffer(
                    mesh_buffers.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );

                let uniforms_offset =
                    i as u32 * self.entity_uniforms.alignment(&self.device) as u32;

                shadow_render_pass.set_bind_group(
                    1,
                    &self.entity_uniforms.bind_group,
                    &[uniforms_offset, uniforms_offset],
                );

                shadow_render_pass.draw_indexed(0..mesh_buffers.index_count, 0, 0..1);
            }
        }

        {
            let mut skybox_render_pass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("skybox render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_texture_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

            skybox_render_pass.set_pipeline(&self.skybox_pipeline);

            skybox_render_pass.set_bind_group(0, &self.skybox_uniforms.bind_group, &[]);
            skybox_render_pass.set_bind_group(1, &self.skybox_texture_bind_group, &[]);

            skybox_render_pass
                .set_vertex_buffer(0, self.builtin_meshes.cube.vertex_buffer.slice(..));
            skybox_render_pass.set_index_buffer(
                self.builtin_meshes.cube.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            skybox_render_pass.draw_indexed(0..self.builtin_meshes.cube.index_count, 0, 0..1);
        }

        {
            let mut surface_render_pass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("surface render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_texture_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

            surface_render_pass.set_pipeline(&self.entity_pipeline);

            surface_render_pass.set_bind_group(0, &self.global_uniforms.bind_group, &[]);
            surface_render_pass.set_bind_group(2, &self.shadow_map_bind_group, &[]);

            for (i, draw_command) in draw_commands.into_iter().enumerate() {
                let Some(mesh_buffers) = self.mesh_buffers.get(&draw_command.mesh_name) else {
                    log::warn!("no mesh exists with name {:?}", draw_command.mesh_name);
                    continue;
                };

                surface_render_pass.set_vertex_buffer(0, mesh_buffers.vertex_buffer.slice(..));
                surface_render_pass.set_index_buffer(
                    mesh_buffers.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );

                let uniforms_offset =
                    i as u32 * self.entity_uniforms.alignment(&self.device) as u32;
                surface_render_pass.set_bind_group(
                    1,
                    &self.entity_uniforms.bind_group,
                    &[uniforms_offset, uniforms_offset],
                );

                surface_render_pass.draw_indexed(0..mesh_buffers.index_count, 0, 0..1);
            }
        }

        self.queue.submit([command_encoder.finish()]);

        self.queue.present(surface_texture);

        Ok(())
    }
}

pub(in crate::app) fn new_depth_texture_view(
    device: &wgpu::Device,
    dimensions: (u32, u32),
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: dimensions.0.max(1),
        height: dimensions.1.max(1),
        depth_or_array_layers: 1,
    };

    let descriptor = wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = device.create_texture(&descriptor);

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(in crate::app) fn new_cubemap_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    paths: &[impl AsRef<Path>; 6],
) -> anyhow::Result<wgpu::TextureView> {
    const CUBEMAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    const BYTES_PER_PIXEL: u32 = 4;

    let mut size: Option<u32> = None;
    let mut faces_rgba8: Vec<Vec<u8>> = Vec::new();

    for p in paths.iter() {
        let p = p.as_ref();
        let img = match image::open(p) {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("failed to open image {}: {e}", p.display())),
        };

        let (w, h) = img.dimensions();

        if w != h {
            return Err(anyhow!(
                "image {} width and height are not equal",
                p.display()
            ));
        }

        if let Some(size) = size {
            if size != w {
                return Err(anyhow!("all images must have the same dimensions"));
            }
        } else {
            size = Some(w);
        }

        let raw = img.to_rgba8().into_raw();

        faces_rgba8.push(raw);
    }

    let size = size.unwrap();
    let bytes_per_row = size * BYTES_PER_PIXEL;

    if !bytes_per_row.is_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) {
        return Err(anyhow!(
            "cubemap size must be a multiple of {}",
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / BYTES_PER_PIXEL
        ));
    }

    let descriptor = wgpu::TextureDescriptor {
        label: Some("cubemap texture"),
        size: wgpu::Extent3d {
            width: size.max(1),
            height: size.max(1),
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CUBEMAP_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = device.create_texture(&descriptor);

    for (i, data) in faces_rgba8.iter().enumerate() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        )
    }

    Ok(texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("cubemap texture view"),
        format: None,
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    }))
}

impl BuiltinMeshes {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            cube: MeshBuffers::from_mesh(Mesh::new_smooth_shaded_cube(), device),
        }
    }
}
