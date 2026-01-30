use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::{Device, TextureFormat};
use winit::{event::WindowEvent, window::Window};

pub struct Gui {
    pub context: egui::Context,
    pub viewport_texture_id: Option<egui::TextureId>,
    state: State,
    renderer: Renderer,
}

impl Gui {
    pub fn new(window: &Window, device: &Device, format: TextureFormat) -> Self {
        let context = egui::Context::default();
        let id = context.viewport_id();
        let state = State::new(context.clone(), id, &window, None, None);
        let renderer = Renderer::new(device, format, None, 1);

        Self {
            context,
            state,
            renderer,
            viewport_texture_id: None,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        // Return 'true' if the UI consumed the event (e.g., clicking a button)
        // to prevent the camera from moving when interacting with the UI.
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn register_viewport_texture(
        &mut self,
        device: &Device,
        texture_view: &wgpu::TextureView,
        _format: TextureFormat,
    ) {
        let id =
            self.renderer
                .register_native_texture(device, texture_view, wgpu::FilterMode::Linear);
        self.viewport_texture_id = Some(id);
    }

    pub fn update_viewport_texture(&mut self, device: &Device, texture_view: &wgpu::TextureView) {
        if let Some(id) = self.viewport_texture_id {
            self.renderer.update_egui_texture_from_wgpu_texture(
                device,
                texture_view,
                wgpu::FilterMode::Linear,
                id,
            );
        }
    }

    pub fn resize(&mut self, _window: &Window) {
        // TODO: Handle resizing if necessary
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        view: &wgpu::TextureView,
        ui_callback: impl FnOnce(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, ui_callback);
        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass); // Unborrow before freeing textures

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
