pub struct Emulator {
    pub renderer: crate::renderer::RendererState,
    pub event_loop: winit::event_loop::EventLoop<()>,
    memory: [u8; 512],
}

impl Emulator {
    pub async fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        let renderer = crate::renderer::RendererState::new(window).await;
        let mut memory = [0x00; 512];
        // TODO: memory font
        Self {
            event_loop,
            renderer,
            memory,
        }
    }
    pub async fn run(mut self) {
        env_logger::init();
        self.event_loop
            .run(move |event, _, control_flow| match event {
                winit::event::Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == self.renderer.window().id() => {
                    if !self.renderer.input(event) {
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
                                self.renderer.resize(*physical_size);
                            }
                            winit::event::WindowEvent::ScaleFactorChanged {
                                new_inner_size,
                                ..
                            } => {
                                self.renderer.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                winit::event::Event::RedrawRequested(window_id)
                    if window_id == self.renderer.window().id() =>
                {
                    self.renderer.update();
                    match self.renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.renderer.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            *control_flow = winit::event_loop::ControlFlow::Exit
                        }
                        Err(e) => eprint!("{:?}", e),
                    }
                }
                winit::event::Event::MainEventsCleared => {
                    self.renderer.window().request_redraw();
                }
                _ => {}
            });
    }
}
