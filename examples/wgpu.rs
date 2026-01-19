use orengine::{CameraUniform, Vertex, load_model};
use wgpu::{RenderPipeline, util::DeviceExt};
use winit::{
    dpi::PhysicalSize, event::*, event_loop::EventLoop, window::Window, window::WindowBuilder,
};

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: std::sync::Arc<Window>,
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    /// The GPU memory block
    index_buffer: wgpu::Buffer,
    /// To know how many points to draw
    num_indices: u32,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    start_time: std::time::Instant, // To animate rotation
}

impl State {
    async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();
        // 1. Instance: Le point d'entrée vers Vulkan/DX12/Metal
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 2. Surface: La zone de dessin sur la fenêtre
        let surface = instance.create_surface(window.clone()).unwrap();

        // 3. Adapter: La carte graphique physique (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // 4. Device & Queue: La connexion logique et la file de commandes
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        // 5. Configuration de la Surface (Format de couleur, VSync, etc.)
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
            present_mode: surface_caps.present_modes[0], // Souvent Fifo (VSync)
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // A. Create the Uniform Data
        let mut camera_uniform = CameraUniform::new();
        // Initial calculation
        camera_uniform.update_view_proj(0.0, config.width as f32 / config.height as f32);

        // B. Create the Buffer (GPU Memory)
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, // COPY_DST allows us to update it later
        });

        // C. Create the Bind Group Layout ( The Interface description )
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,                             // Slot 0
                    visibility: wgpu::ShaderStages::VERTEX, // Only Vertex shader needs it
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        // D. Create the Bind Group ( The Actual Connection )
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let (model_vertices, model_indices) = load_model("pizza.obj");

        // 1. Vertex Buffer avec les données du modèle
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&model_vertices), // Utilise le vecteur chargé
            usage: wgpu::BufferUsages::VERTEX,
        });

        // 2. Create Index Buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&model_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = model_indices.len() as u32;

        // 2. Define the Vertex Buffer Layout
        // This tells wgpu how to read the bytes.
        // "Hey GPU, read 24 bytes at a time. The first 12 bytes are Position, the next 12 are Color."
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // How wide is one vertex?
            step_mode: wgpu::VertexStepMode::Vertex, // Advance by vertex, not instance
            attributes: &[
                // Attribute 0: Position (Offset 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // Corresponds to @location(0) in shader
                    format: wgpu::VertexFormat::Float32x3, // vec3<f32>
                },
                // Attribute 1: Color (Offset 12 bytes - because 3 * 4 bytes per float)
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // Corresponds to @location(1) in shader
                    format: wgpu::VertexFormat::Float32x3, // vec3<f32>
                },
            ],
        };

        // 1. Charger le shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

        // 2. Créer le Layout (la description des inputs, vide pour l'instant)
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        // 3. Créer le Pipeline (L'objet lourd qui contient tout)
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // Nom de la fonction dans wgsl
                buffers: &[vertex_buffer_layout],
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
                topology: wgpu::PrimitiveTopology::TriangleList, // On dessine des triangles
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Sens anti-horaire
                cull_mode: Some(wgpu::Face::Back), // Ne pas dessiner le dos du triangle
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // Pas de Z-buffer pour l'instant
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            start_time: std::time::Instant::now(),
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        // Calculate new rotation based on time
        let time = self.start_time.elapsed().as_secs_f32();
        let aspect = self.config.width as f32 / self.config.height as f32;

        // Recalculate the matrix logic
        self.camera_uniform.update_view_proj(time, aspect);

        // Send the new data to the GPU
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // A. On récupère la prochaine texture libre pour dessiner
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // B. On crée un encodeur de commandes pour le GPU
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // C. RenderPass: On commence à dessiner
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Couleur de fond (R, G, B, A) - Ici un bleu "Tunic-style"
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // Pas de 3D (Z-buffer) pour l'instant
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Plug in the uniform data
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // 1. Bind Vertex Buffer (Slot 0)
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // 2. Bind Index Buffer (NEW)
            // We must specify the format (Uint16 because our array is u16)
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // 3. Draw Indexed (NEW)
            // ranges: indices, base_vertex, instances
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // D. On envoie le tout au GPU
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = std::sync::Arc::new(
        WindowBuilder::new()
            .with_title("Orengine")
            .build(&event_loop)
            .unwrap(),
    );

    // Initialisation asynchrone via pollster
    let mut state = pollster::block_on(State::new(window.clone()));
    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                WindowEvent::RedrawRequested => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => state.window.request_redraw(),
            _ => {}
        })
        .unwrap();
}
