pub async fn run() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut state = crate::state::State::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,
                    winit::event::WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        winit::event::Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                Err(e) => eprint!("{:?}", e),
            }
        }
        winit::event::Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        _ => {}
    });
}
