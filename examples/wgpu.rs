use orengine::State;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = std::sync::Arc::new(
        WindowBuilder::new()
            .with_title("Orengine")
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let mut state = match pollster::block_on(State::new(window.clone(), "drone_costum.obj")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create Orengine state: {}", e);
            // We can't recover from this, so exit
            std::process::exit(1);
        }
    };

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
                        Err(orengine::error::OrengineError::SurfaceError(
                            wgpu::SurfaceError::OutOfMemory,
                        )) => target.exit(),
                        Err(orengine::error::OrengineError::SurfaceError(_)) => {
                            state.resize(state.size)
                        }
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
                state.handle_mouse_motion(delta);
            }
            Event::AboutToWait => state.window.request_redraw(),
            _ => {}
        })
        .unwrap();
}
