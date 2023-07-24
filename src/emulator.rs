use std::time::Duration;
use winit::event::VirtualKeyCode;

const FPS: f64 = 60.0;

pub struct Emulator {
    pub renderer: crate::renderer::RendererState,
    pub event_loop: winit::event_loop::EventLoop<()>,
    memory: [u8; 4096],
    stack: crate::stack::Stack,
    _delay_timer: crate::timer::Timer,
    _sound_timer: crate::timer::Timer,
    program_counter: u16,
    _variable_registers: [bool; 16],
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

        let _stack = crate::stack::Stack::new();
        let _delay_timer = crate::timer::Timer::new();
        let _sound_timer = crate::timer::Timer::new();
        let program_counter = 0;
        let _variable_registers = [false; 16];

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
            memory,
            stack: _stack,
            _delay_timer,
            _sound_timer,
            program_counter,
            _variable_registers,
            pressed,
        }
    }

    pub async fn run(mut self) {
        env_logger::init();
        self.event_loop.run(move |event, _, control_flow| {
            let start_frame = std::time::Instant::now();

            // input
            match event {
                winit::event::Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == self.renderer.window().id() => match event {
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } if self.pressed.contains_key(&keycode) => {
                        *self.pressed.get_mut(keycode).unwrap() = true;
                    }
                    _ => {}
                },
                _ => {}
            };

            // 12 instructions per frame = 720 instructions per second given 60 FPS
            for _ in 0..12 {
                // fetch
                let read_location = self.program_counter as usize;
                let larger_byte = self.memory[read_location];
                let smaller_byte = self.memory[read_location + 1];
                let parsed_instruction = combine_bytes_into_instruction(larger_byte, smaller_byte);

                // decode and execute
                self.execute_instruction(parsed_instruction);
            }

            // render
            self.renderer.run(event, control_flow);

            // key state reset after every frame
            for value in self.pressed.values_mut() {
                *value = false;
            }

            // sleep to maintain FPS
            let time_passed = (1_000_000_000.0 / FPS) - start_frame.elapsed().as_nanos() as f64;
            spin_sleep::sleep(Duration::new(0, time_passed as u32));
        });
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

    fn execute_instruction(&mut self, parsed_instruction: u16) {
        let first_nibble =
            crate::bit_utils::get_u16_bit_range_as_number(parsed_instruction, 12, 16).unwrap();
        let second_nibble =
            crate::bit_utils::get_u16_bit_range_as_number(parsed_instruction, 8, 12).unwrap();
        let third_nibble =
            crate::bit_utils::get_u16_bit_range_as_number(parsed_instruction, 4, 8).unwrap();
        let fourth_nibble =
            crate::bit_utils::get_u16_bit_range_as_number(parsed_instruction, 0, 4).unwrap();
        let memory_address =
            crate::bit_utils::get_u16_bit_range_as_number(parsed_instruction, 4, 16).unwrap();
        let two_nibble_argument =
            crate::bit_utils::get_u16_bit_range_as_number(parsed_instruction, 8, 18).unwrap();

        match first_nibble {
            0x0 => match fourth_nibble {
                0xE => self.return_from_stack(),
                _ => {}
            },
            _ => {}
        };
    }

    pub fn return_from_stack(&mut self) {
        self.program_counter = self.stack.pop().unwrap();
    }
}

fn combine_bytes_into_instruction(larger_byte: u8, smaller_byte: u8) -> u16 {
    let mut result: u16 = 0;
    for i in 8..16 {
        result = crate::bit_utils::set_bit_from_other_number(larger_byte.into(), i - 8, result, i);
    }

    for i in 0..8 {
        result = crate::bit_utils::set_bit_from_other_number(smaller_byte.into(), i, result, i);
    }

    result
}

#[test]
pub fn test_combine_bytes_into_instruction() {
    assert_eq!(
        combine_bytes_into_instruction(0b00000100, 0b00100000),
        0b0000010000100000
    );
}
