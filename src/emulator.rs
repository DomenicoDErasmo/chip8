use winit::event::VirtualKeyCode;

const FPS: f64 = 60.0;

pub struct Emulator {
    memory: [u8; 4096],
    _stack: crate::stack::Stack,
    _delay_timer: crate::timer::Timer,
    _sound_timer: crate::timer::Timer,
    pressed: std::collections::HashMap<VirtualKeyCode, bool>,
    program_counter: usize,
    index_register: usize,
    variable_registers: [u8; 16],
    screen: [[bool; crate::screen::SCREEN_WIDTH as usize]; crate::screen::SCREEN_HEIGHT as usize],
}

impl Emulator {
    pub async fn new() -> Self {
        let mut memory = [0x00; 4096];
        Self::load_font(&mut memory);

        let stack = crate::stack::Stack::new();
        let delay_timer = crate::timer::Timer::new();
        let sound_timer = crate::timer::Timer::new();

        let program_counter = 0;
        let index_register = 0;

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

        let variable_registers = [0; 16];

        let screen =
            [[false; crate::screen::SCREEN_WIDTH as usize]; crate::screen::SCREEN_HEIGHT as usize];

        Self {
            memory,
            _stack: stack,
            _delay_timer: delay_timer,
            _sound_timer: sound_timer,
            pressed,
            program_counter,
            index_register,
            variable_registers,
            screen,
        }
    }

    pub async fn run(mut self) {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new();
        let window: winit::window::Window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        let mut renderer = crate::renderer::RendererState::new(window).await;
        let mut i = 0;
        let timer_length = std::time::Duration::new(0, (1_000_000_000.0 / FPS) as u32);

        let mut x: i32 = 0;
        let mut y: i32 = 0;

        let mut prev_x: i32 = 0;
        let mut prev_y: i32 = 0;

        event_loop.run(move |event, _, control_flow| match event {
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
            } if window_id == renderer.window().id() => match event {
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
                    renderer.resize(*physical_size);
                }
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size);
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
                        match keycode {
                            VirtualKeyCode::W
                            | VirtualKeyCode::A
                            | VirtualKeyCode::S
                            | VirtualKeyCode::D => {
                                prev_x = x;
                                prev_y = y;
                                match keycode {
                                    VirtualKeyCode::W => {
                                        y = y + 1 % crate::screen::SCREEN_HEIGHT as i32
                                    }
                                    VirtualKeyCode::A => {
                                        x = x - 1 % crate::screen::SCREEN_WIDTH as i32
                                    }
                                    VirtualKeyCode::S => {
                                        y = y - 1 % crate::screen::SCREEN_HEIGHT as i32
                                    }
                                    VirtualKeyCode::D => {
                                        x = x + 1 % crate::screen::SCREEN_WIDTH as i32
                                    }
                                    _ => {}
                                };
                                renderer.instances
                                    [(y * crate::screen::SCREEN_WIDTH as i32 + x) as usize]
                                    .color = cgmath::Vector4 {
                                    x: 1.0,
                                    y: 1.0,
                                    z: 0.0,
                                    w: 0.0,
                                };
                                println!(
                                    "prev coords: {prev_y}, {prev_x}, z: {}",
                                    renderer.instances[(prev_y * crate::screen::SCREEN_WIDTH as i32
                                        + prev_x)
                                        as usize]
                                        .color
                                        .z
                                );
                                let new_color = if renderer.instances[(prev_y
                                    * crate::screen::SCREEN_WIDTH as i32
                                    + prev_x)
                                    as usize]
                                    .color
                                    .z
                                    > 0.0
                                {
                                    0.0
                                } else {
                                    1.0
                                };
                                renderer.instances[(prev_y * crate::screen::SCREEN_WIDTH as i32
                                    + prev_x)
                                    as usize]
                                    .color = cgmath::Vector4 {
                                    x: new_color,
                                    y: new_color,
                                    z: new_color,
                                    w: 0.0,
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            // explicit redraw request
            winit::event::Event::RedrawRequested(window_id)
                if window_id == renderer.window().id() =>
            {
                renderer.update();
                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    Err(e) => eprint!("{:?}", e),
                }
            }
            // everything else - no input or waiting
            winit::event::Event::MainEventsCleared => {
                // 12x a frame -> 720 / instructions per second on 60 FPS
                for _ in 0..12 {
                    // fetch
                    let instruction_bytes: &[u8; 2] = &self.memory
                        [self.program_counter..self.program_counter + 2]
                        .try_into()
                        .expect("Wrong length");

                    // decode
                    let instruction = crate::bit_utils::append_number_bits(instruction_bytes);
                    self.parse_instruction(instruction, &mut renderer);
                    // execute
                }

                // timer test
                // TODO: remove
                i = i + 1;
                if i >= FPS as i32 {
                    i = 0;
                    println!("A second passed");
                }

                // render
                renderer.window().request_redraw();
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

    fn parse_instruction(
        &mut self,
        instruction: u16,
        renderer: &mut crate::renderer::RendererState,
    ) {
        let first_nibble = crate::bit_utils::bit_range_to_num(instruction, 0, 4).unwrap();
        let second_nibble = crate::bit_utils::bit_range_to_num(instruction, 4, 8).unwrap();
        let third_nibble = crate::bit_utils::bit_range_to_num(instruction, 8, 12).unwrap();
        let fourth_nibble = crate::bit_utils::bit_range_to_num(instruction, 12, 16).unwrap();
        let nibbles_2_to_4 = crate::bit_utils::bit_range_to_num(instruction, 4, 16).unwrap();
        let nibbles_3_to_4 = crate::bit_utils::bit_range_to_num(instruction, 8, 16).unwrap();

        match first_nibble {
            0x0 => match fourth_nibble {
                0x0 => self.clear_screen(),
                0xE => {}
                _ => {}
            },
            0x1 => self.program_counter = nibbles_2_to_4.into(),
            0x2 => {}
            0x3 => {}
            0x4 => {}
            0x5 => {}
            0x6 => self.variable_registers[second_nibble as usize] = nibbles_3_to_4 as u8,
            0x7 => {
                // normally we would check if <= 255 but Rust does this for us
                let register_to_change = &mut self.variable_registers[second_nibble as usize];
                let candidate_sum = *register_to_change + (nibbles_3_to_4 as u8);
                *register_to_change = candidate_sum;
            }
            0x8 => {}
            0x9 => {}
            0xA => self.index_register = nibbles_3_to_4.into(),
            0xB => {}
            0xC => {}
            0xD => self.draw_to_screen(
                second_nibble as usize,
                third_nibble as usize,
                fourth_nibble as usize,
                renderer,
            ),
            0xE => {}
            0xF => {}
            _ => {}
        }
    }

    fn draw_to_screen(
        &mut self,
        _second_nibble: usize,
        _third_nibble: usize,
        _fourth_nibble: usize,
        renderer: &mut crate::renderer::RendererState,
    ) {
        (*renderer).update();
    }

    fn clear_screen(&mut self) {
        for row in self.screen.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
    }
}
