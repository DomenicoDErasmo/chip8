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
}

impl Emulator {
    pub async fn new(file_path: Option<&str>) -> Self {
        let mut memory = match file_path {
            Some(path) => Self::load_memory_from_rom(path),
            None => [0; 4096],
        };
        Self::load_font(&mut memory);

        let stack = crate::stack::Stack::new();
        let delay_timer = crate::timer::Timer::new();
        let sound_timer = crate::timer::Timer::new();

        let program_counter = 0x200;
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

        Self {
            memory,
            _stack: stack,
            _delay_timer: delay_timer,
            _sound_timer: sound_timer,
            pressed,
            program_counter,
            index_register,
            variable_registers,
        }
    }

    pub async fn run(mut self) {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new();
        let window: winit::window::Window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        let mut renderer = crate::renderer::RendererState::new(window).await;
        renderer.instances[(9 * crate::screen::SCREEN_WIDTH + 12) as usize].color =
            cgmath::Vector4 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
                w: 0.0,
            };
        let timer_length = std::time::Duration::new(0, (1_000_000_000.0 / FPS) as u32);

        let mut x: i32 = 0;
        let mut y: i32 = 0;

        let mut _prev_x: i32 = 0;
        let mut _prev_y: i32 = 0;

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
                        // Just for testing TODO: remove
                        match keycode {
                            VirtualKeyCode::W
                            | VirtualKeyCode::A
                            | VirtualKeyCode::S
                            | VirtualKeyCode::D => {
                                _prev_x = x;
                                _prev_y = y;
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
                                    "prev coords: {_prev_y}, {_prev_x}, z: {}",
                                    renderer.instances[(_prev_y
                                        * crate::screen::SCREEN_WIDTH as i32
                                        + _prev_x)
                                        as usize]
                                        .color
                                        .z
                                );
                                let new_color = if renderer.instances[(_prev_y
                                    * crate::screen::SCREEN_WIDTH as i32
                                    + _prev_x)
                                    as usize]
                                    .color
                                    .z
                                    > 0.0
                                {
                                    0.0
                                } else {
                                    1.0
                                };
                                renderer.instances[(_prev_y * crate::screen::SCREEN_WIDTH as i32
                                    + _prev_x)
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

                    // increment program counter for next instruction
                    self.program_counter = self.program_counter + 2;
                    println!("program_counter: {:#?}", self.program_counter);

                    println!("instruction_bytes: {instruction_bytes:#?}");

                    // decode
                    self.parse_instruction(instruction_bytes, &mut renderer);
                    // execute
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
        instruction: &[u8; 2],
        renderer: &mut crate::renderer::RendererState,
    ) {
        let first_nibble = crate::bit_utils::bit_range_to_num(instruction[0].into(), 4, 8).unwrap();
        let second_nibble =
            crate::bit_utils::bit_range_to_num(instruction[0].into(), 0, 4).unwrap();
        let third_nibble = crate::bit_utils::bit_range_to_num(instruction[1].into(), 4, 8).unwrap();
        let fourth_nibble =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 0, 4).unwrap();
        let nibbles_3_to_4 =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 0, 8).unwrap();
        let nibbles_2_to_4 =
            crate::bit_utils::append_number_bits(&[second_nibble as u8, nibbles_3_to_4 as u8]);

        match first_nibble {
            0x0 => match fourth_nibble {
                0x0 => clear_screen(renderer),
                0xE => {}
                _ => {}
            },
            0x1 => self.jump(nibbles_2_to_4.into()),
            0x2 => {}
            0x3 => {}
            0x4 => {}
            0x5 => {}
            0x6 => self.set_register(second_nibble as usize, nibbles_3_to_4 as u8),
            0x7 => self.add_to_register(second_nibble as usize, nibbles_3_to_4 as u8),
            0x8 => {}
            0x9 => {}
            0xA => self.set_index_register(nibbles_2_to_4.into()),
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

    fn jump(&mut self, location: usize) {
        self.program_counter = location;
    }

    fn draw_to_screen(
        &mut self,
        second_nibble: usize,
        third_nibble: usize,
        fourth_nibble: usize,
        renderer: &mut crate::renderer::RendererState,
    ) {
        let mut x = self.variable_registers[second_nibble] % 64;
        let mut y = self.variable_registers[third_nibble] % 32;

        self.variable_registers[15] = 0;

        'outer: for i in 0..fourth_nibble {
            let ith_byte = self.memory[self.index_register + i];
            'inner: for bit_value in 0..8 {
                let sprite_bit =
                    crate::bit_utils::bit_range_to_num(ith_byte as u16, bit_value, bit_value + 1)
                        .unwrap();
                let pixel_index = (y as u32 * crate::screen::SCREEN_WIDTH + x as u32) as usize;
                let pixel = &mut renderer.instances[pixel_index];

                if sprite_bit == 1 {
                    if pixel.color.x > 0.1 {
                        pixel.color = cgmath::Vector4 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            w: 0.0,
                        };
                        self.variable_registers[15] = 1;
                    } else {
                        pixel.color = cgmath::Vector4 {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                            w: 0.0,
                        };
                    }
                }

                x = x + 1;
                if x >= crate::screen::SCREEN_WIDTH as u8 {
                    break 'inner;
                }
            }
            y = y + 1;
            if y >= crate::screen::SCREEN_HEIGHT as u8 {
                break 'outer;
            }
        }
    }

    fn set_register(&mut self, register: usize, value: u8) {
        self.variable_registers[register] = value;
    }

    fn add_to_register(&mut self, register: usize, addend: u8) {
        let register_to_change = &mut self.variable_registers[register];
        let sum = *register_to_change + addend;
        *register_to_change = sum;
    }

    fn set_index_register(&mut self, address: usize) {
        self.index_register = address;
    }

    fn load_memory_from_rom(file_path: &str) -> [u8; 4096] {
        let mut memory = [0; 4096];

        let rom_contents = std::fs::read(file_path).unwrap();
        for (i, instruction) in rom_contents.iter().enumerate() {
            memory[0x200 + i] = *instruction;
        }

        memory
    }
}

fn clear_screen(renderer: &mut crate::renderer::RendererState) {
    for instance in renderer.instances.iter_mut() {
        instance.color = cgmath::Vector4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
    }
}

#[tokio::test]
async fn test_jump() {
    let mut emulator = Emulator::new(None).await;
    emulator.jump(0x210);
    assert_eq!(emulator.program_counter, 0x210);
}

#[tokio::test]
async fn test_set_register() {
    let mut emulator = Emulator::new(None).await;
    emulator.set_register(12, 41);
    assert_eq!(emulator.variable_registers[12], 41);
}

#[tokio::test]
async fn test_add_to_register() {
    let mut emulator = Emulator::new(None).await;
    emulator.add_to_register(11, 15);
    assert_eq!(emulator.variable_registers[11], 15);
    emulator.add_to_register(11, 215);
    assert_eq!(emulator.variable_registers[11], 230);
}

#[tokio::test]
async fn test_set_index_register() {
    let mut emulator = Emulator::new(None).await;
    emulator.set_index_register(1411);
    assert_eq!(emulator.index_register, 1411);
}
