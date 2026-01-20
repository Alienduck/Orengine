use orengine::State;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = std::sync::Arc::new(
        WindowBuilder::new()
            .with_title("Orengine")
            .build(&event_loop)
            .unwrap(),
    );

    // We pass the .obj file here! No more hardcoding in the engine.
    let mut state = pollster::block_on(State::new(window.clone(), "pizza.obj"));
    let mut right_mouse_pressed = false;

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => match event {
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Right,
                    ..
                } => {
                    right_mouse_pressed = true;
                    // https://github.com/rust-windowing/winit/issues/1677
                    let _ = state
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::Confined);
                    state.window.set_cursor_visible(false);
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Right,
                    ..
                } => {
                    right_mouse_pressed = false;
                    let _ = state
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::None);
                    state.window.set_cursor_visible(true);
                }
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
                ref event if state.input(event) => {}
                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if right_mouse_pressed {
                    state.handle_mouse_motion(delta);
                }
            }
            Event::AboutToWait => state.window.request_redraw(),
            _ => {}
        })
        .unwrap();
}
