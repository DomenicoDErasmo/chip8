use std::time::Duration;

use winit::event::VirtualKeyCode;

const FPS: f64 = 60.0;

pub struct Emulator {
    pub renderer: crate::renderer::RendererState,
    pub event_loop: winit::event_loop::EventLoop<()>,
    _memory: [u8; 4096],
    _stack: crate::stack::Stack,
    _delay_timer: crate::timer::Timer,
    _sound_timer: crate::timer::Timer,
    _program_counter: u16,
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
        let _program_counter = 0;
        let _variable_registers = [false; 16];

        let _pressed = std::collections::HashMap::from([
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
            _stack,
            _delay_timer,
            _sound_timer,
            _program_counter,
            _variable_registers,
            pressed: _pressed,
        }
    }

    pub async fn run(mut self) {
        env_logger::init();
        let mut i = 0;
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

            for _ in 0..12 {
                // fetch
                // decode
                // execute
            }

            // render
            self.renderer.run(event, control_flow);

            // TODO: state reset - set keys to unpressed (should I do this elsewhere? do I need this?)

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
}
