use crate::{
    camera::{Camera, CameraUniform},
    error::{OrengineError, Result},
    gui::Gui,
    input::InputHandler,
    instance::{Instance, InstanceRaw},
    light::LightUniform,
    models::load_model,
    textures,
    vertex::Vertex,
};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct MeshRenderData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material_id: usize,
}

pub struct MaterialRenderData {
    pub bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub texture: textures::Texture,
}

/// The main state of the application, holding all WGPU and rendering data.
/// This struct is responsible for managing the GPU resources, rendering pipeline,
/// and handling the rendering loop.
pub struct State {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: std::sync::Arc<Window>,
    pub gui: Gui,
    pub light_uniform: LightUniform,

    render_pipeline: wgpu::RenderPipeline,
    render_target: textures::Texture,
    meshes: Vec<MeshRenderData>,
    materials: Vec<MaterialRenderData>,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    camera: Camera,
    input_handler: InputHandler,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    #[allow(dead_code)]
    depth_texture: textures::Texture,

    is_scene_hovered: bool,

    #[allow(dead_code)]
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
}

impl State {
    // We pass the mode path as parameter now
    pub async fn new(window: std::sync::Arc<Window>, model_path: &str) -> Result<Self> {
        let size = window.inner_size();

        // 1. Instance & Surface
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        // 2. Adapte Device & Queue
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(OrengineError::NoGpuAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // 3. Config
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // 4. Assets (Model & Textures)
        let model = load_model(model_path)?;

        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: glam::Vec3 = glam::Vec3::new(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = glam::Vec3::new(x as f32 * 3.0, 0.0, z as f32 * 3.0)
                        - INSTANCE_DISPLACEMENT;

                    let rotation = if position == glam::Vec3::ZERO {
                        glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0)
                    } else {
                        glam::Quat::from_axis_angle(position.normalize(), 45.0f32.to_radians())
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // 6. Camera
        let camera = Camera {
            eye: (0.0, 1.0, 5.0).into(),
            target: (0.0, 1.0, 0.0).into(),
            up: glam::Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 100.0,
        };

        let input_handler = InputHandler::new(0.01);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // 7. Texture Bind Group Layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // Process Materials
        let mut materials = Vec::new();
        for mat in &model.materials {
            let texture_path = std::path::Path::new("assets").join(&mat.diffuse_texture);

            let texture = if !mat.diffuse_texture.is_empty() {
                textures::Texture::from_image(&device, &queue, &texture_path, Some(&mat.name))
                    .unwrap_or_else(|_| {
                        eprintln!(
                            "Erreur chargement texture: {:?}. Utilisation texture magenta.",
                            texture_path
                        );
                        textures::Texture::from_color(
                            &device,
                            &queue,
                            [255, 0, 255, 255],
                            Some(&mat.name),
                        )
                    })
            } else {
                textures::Texture::from_color(
                    &device,
                    &queue,
                    [255, 255, 255, 255],
                    Some(&mat.name),
                )
            };

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some(&mat.name),
            });

            materials.push(MaterialRenderData {
                bind_group,
                texture,
            });
        }

        // Process Meshes
        let meshes = model
            .meshes
            .iter()
            .map(|m| {
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", m.name)),
                    contents: bytemuck::cast_slice(&m.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", m.name)),
                    contents: bytemuck::cast_slice(&m.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                MeshRenderData {
                    vertex_buffer,
                    index_buffer,
                    num_elements: m.indices.len() as u32,
                    material_id: m.material_id,
                }
            })
            .collect::<Vec<_>>();

        // 8. Depth Texture
        let depth_texture =
            textures::Texture::create_depth_texture(&device, &config, "depth_texture");

        let light_uniform = crate::light::LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 0.0, 0.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("light_bind_group_layout"),
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });

        // 9. Pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
                format: crate::textures::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let render_target =
            crate::textures::Texture::create_render_target(&device, &config, "Render Target");

        let mut gui = Gui::new(&window, &device, config.format);

        gui.register_viewport_texture(&device, &render_target.view, config.format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            render_target,
            meshes,
            materials,
            camera,
            input_handler,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            is_scene_hovered: false,
            instances,
            instance_buffer,
            light_uniform,
            light_buffer,
            light_bind_group,
            gui,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.render_target = crate::textures::Texture::create_render_target(
                &self.device,
                &self.config,
                "Render Target",
            );
            self.depth_texture = textures::Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
            );
            self.gui
                .update_viewport_texture(&self.device, &self.render_target.view);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let consumed = self.gui.handle_event(&self.window, event);

        let handled =
            self.input_handler
                .process_input(event, &self.window, consumed, self.is_scene_hovered);

        consumed || handled
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.input_handler.handle_mouse_motion(delta);
    }

    pub fn update(&mut self) {
        self.input_handler
            .camera_controller
            .update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view_surface = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            for mesh in &self.meshes {
                let material = &self.materials[mesh.material_id];
                render_pass.set_bind_group(1, &material.bind_group, &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.instances.len() as _);
            }
        }

        let texture_id = self.gui.viewport_texture_id;

        let mut temp_light_position = self.light_uniform.position;
        let mut temp_light_color = self.light_uniform.color;

        let mut is_scene_hovered = self.is_scene_hovered;

        self.gui.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view_surface,
            |ctx| {
                egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("Fichier", |_| {});
                    });
                });
                egui::SidePanel::left("hierarchy").show(ctx, |ui| {
                    ui.label("Scène 3D");
                    ui.separator();
                    ui.label("Pizzas (x100)");
                });

                egui::SidePanel::right("inspector").show(ctx, |ui| {
                    ui.heading("Lumière");
                    ui.add(egui::Slider::new(&mut temp_light_position[0], -10.0..=10.0).text("X"));
                    ui.add(egui::Slider::new(&mut temp_light_position[1], -10.0..=10.0).text("Y"));
                    ui.add(egui::Slider::new(&mut temp_light_position[2], -10.0..=10.0).text("Z"));

                    ui.separator();
                    ui.label("Couleur");
                    ui.color_edit_button_rgb(&mut temp_light_color);
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(id) = texture_id {
                        let response =
                            ui.image(egui::load::SizedTexture::new(id, ui.available_size()));
                        is_scene_hovered = response.hovered();
                    } else {
                        ui.label("Chargement de la texture...");
                    }
                });
            },
        );

        self.is_scene_hovered = is_scene_hovered;

        self.light_uniform.position = temp_light_position;
        self.light_uniform.color = temp_light_color;

        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
