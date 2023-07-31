use winit::event::VirtualKeyCode;

const FPS: f64 = 60.0;

pub struct Emulator {
    pub renderer: crate::renderer::RendererState,
    pub event_loop: winit::event_loop::EventLoop<()>,
    _memory: [u8; 4096],
    _stack: crate::stack::Stack,
    _delay_timer: crate::timer::Timer,
    _sound_timer: crate::timer::Timer,
    pressed: std::collections::HashMap<VirtualKeyCode, bool>,
}

impl Emulator {
    pub async fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        let renderer = crate::renderer::RendererState::new(window).await;
        let mut memory = [0x00; 4096];
        Self::load_font(&mut memory);

        let stack = crate::stack::Stack::new();
        let delay_timer = crate::timer::Timer::new();
        let sound_timer = crate::timer::Timer::new();

        let pressed = std::collections::HashMap::from([
            (VirtualKeyCode::Key1, false),
            (VirtualKeyCode::Key2, false),
            (VirtualKeyCode::Key3, false),
            (VirtualKeyCode::Key4, false),
            (VirtualKeyCode::Q, false),
            (VirtualKeyCode::W, false),
            (VirtualKeyCode::E, false),
            (VirtualKeyCode::R, false),
            (VirtualKeyCode::A, false),
            (VirtualKeyCode::S, false),
            (VirtualKeyCode::D, false),
            (VirtualKeyCode::F, false),
            (VirtualKeyCode::Z, false),
            (VirtualKeyCode::X, false),
            (VirtualKeyCode::C, false),
            (VirtualKeyCode::V, false),
        ]);

        Self {
            event_loop,
            renderer,
            _memory: memory,
            _stack: stack,
            _delay_timer: delay_timer,
            _sound_timer: sound_timer,
            pressed,
        }
    }

    pub async fn run(mut self) {
        env_logger::init();
        let mut i = 0;
        let timer_length = std::time::Duration::new(0, (1_000_000_000.0 / FPS) as u32);
        self.event_loop
            .run(move |event, _, control_flow| match event {
                // wait a frame on init
                winit::event::Event::NewEvents(winit::event::StartCause::Init) => {
                    control_flow.set_wait_until(std::time::Instant::now() + timer_length);
                }
                // wait a frame
                winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                    ..
                }) => {
                    control_flow.set_wait_until(std::time::Instant::now() + timer_length);
                }
                // resizing window, closing window, user input
                winit::event::Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == self.renderer.window().id() => match event {
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,
                    winit::event::WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(*physical_size);
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.renderer.resize(**new_inner_size);
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => match virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Escape) => {
                            *control_flow = winit::event_loop::ControlFlow::Exit
                        }
                        Some(keycode) if self.pressed.contains_key(keycode) => {
                            println!("{:?} was pressed", keycode);
                            *self.pressed.get_mut(keycode).unwrap() = true;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                // explicit redraw request
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
                // everything else - no input or waiting
                winit::event::Event::MainEventsCleared => {
                    // fetch, decode, execute 12x a frame
                    for _ in 0..12 {}

                    // timer test
                    // TODO: remove
                    i = i + 1;
                    if i >= FPS as i32 {
                        i = 0;
                        println!("A second passed");
                    }

                    // render
                    self.renderer.window().request_redraw();
                }
                _ => {}
            })
    }

    fn load_font(memory: &mut [u8; 4096]) {
        let font = &[
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        memory[0x50..0xA0].clone_from_slice(font);
    }
}
