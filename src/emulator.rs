const FPS: f64 = 60.0;

pub struct Emulator {
    memory: [u8; 4096],
    stack: crate::stack::Stack,
    delay_timer: crate::timer::Timer,
    sound_timer: crate::timer::Timer,
    pressed: std::collections::HashMap<u8, bool>,
    pressed_hex_map: std::collections::HashMap<u8, u8>,
    program_counter: usize,
    index_register: usize,
    registers: [u8; 16],
    has_cosmac_vip_instructions: bool,
}

impl Emulator {
    pub async fn new(file_path: Option<&str>, has_cosmac_vip_instructions: bool) -> Self {
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
            // 1234
            (2, false),
            (3, false),
            (4, false),
            (5, false),
            // QWER
            (17, false),
            (18, false),
            (19, false),
            (20, false),
            // ASDF
            (31, false),
            (32, false),
            (33, false),
            (34, false),
            // ZXCV
            (46, false),
            (47, false),
            (48, false),
            (49, false),
        ]);

        // COSMAC VIP Layout:
        // 1 2 3 C
        // 4 5 6 D
        // 7 8 9 E
        // A 0 B F
        //
        // QWERTY Keyboard Layout:
        // 1 2 3 4
        // Q W E R
        // A S D F
        // Z X C V
        //
        // Mapped to scancodes:
        //
        //  2  3  4  5
        // 17 18 19 20
        // 31 32 33 34
        // 46 47 48 49
        let pressed_hex_map = std::collections::HashMap::from([
            (0x0, 47),
            (0x1, 2),
            (0x2, 3),
            (0x3, 4),
            (0x4, 17),
            (0x5, 18),
            (0x6, 19),
            (0x7, 31),
            (0x8, 32),
            (0x9, 33),
            (0xA, 46),
            (0xB, 48),
            (0xC, 5),
            (0xD, 20),
            (0xE, 34),
            (0xF, 49),
        ]);

        let registers = [0; 16];

        Self {
            memory,
            stack,
            delay_timer,
            sound_timer,
            pressed,
            pressed_hex_map,
            program_counter,
            index_register,
            registers,
            has_cosmac_vip_instructions,
        }
    }

    pub async fn run(mut self) {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new();
        let window: winit::window::Window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        let mut renderer = crate::renderer::RendererState::new(window).await;
        let timer_length = std::time::Duration::new(0, (1_000_000_000.0 / FPS) as u32);

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
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
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
                            scancode,
                            ..
                        },
                    ..
                } => {
                    let casted_scancode = &(*scancode as u8);
                    if self.pressed.contains_key(casted_scancode) {
                        *self.pressed.get_mut(casted_scancode).unwrap() = true;
                    }
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            state: winit::event::ElementState::Released,
                            scancode,
                            ..
                        },
                    ..
                } => {
                    let casted_scancode = &(*scancode as u8);
                    if self.pressed.contains_key(casted_scancode) {
                        *self.pressed.get_mut(casted_scancode).unwrap() = false;
                    }
                }
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
            crate::bit_utils::bit_range_to_num(instruction[0].into(), 0, 4).unwrap() as usize;
        let third_nibble =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 4, 8).unwrap() as usize;
        let fourth_nibble =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 0, 4).unwrap();
        let nibbles_3_to_4 =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 0, 8).unwrap() as u8;
        let nibbles_2_to_4 =
            crate::bit_utils::append_number_bits(&[second_nibble as u8, nibbles_3_to_4 as u8])
                as usize;

        match first_nibble {
            0x0 => match fourth_nibble {
                0x0 => clear_screen(renderer),
                0xE => self.stack_return(),
                _ => {}
            },
            0x1 => self.jump(nibbles_2_to_4),
            0x2 => self.call_subroutine(nibbles_2_to_4),
            0x3 => self.skip_if_register_equals_value(second_nibble, nibbles_3_to_4),
            0x4 => self.skip_if_register_not_equal_to_value(second_nibble, nibbles_3_to_4),
            0x5 => self.skip_if_registers_equal(second_nibble, third_nibble),
            0x6 => self.set_register(second_nibble as usize, nibbles_3_to_4),
            0x7 => self.add_to_register(second_nibble as usize, nibbles_3_to_4),
            0x8 => match fourth_nibble {
                0x0 => self.set_register_to_other(second_nibble, third_nibble),
                0x1 => self.binary_or(second_nibble, third_nibble),
                0x2 => self.binary_and(second_nibble, third_nibble),
                0x3 => self.binary_xor(second_nibble, third_nibble),
                0x4 => self.add_registers(second_nibble, third_nibble),
                0x5 => self.subtract_registers(second_nibble, third_nibble),
                0x6 => self.right_shift_on_register(second_nibble, third_nibble),
                0x7 => self.subtract_registers(third_nibble, second_nibble),
                0xE => self.left_shift_on_register(second_nibble, third_nibble),
                _ => {}
            },
            0x9 => self.skip_if_registers_not_equal(second_nibble, third_nibble),
            0xA => self.set_index_register(nibbles_2_to_4),
            0xB => match self.has_cosmac_vip_instructions {
                true => self.jump_with_offset(None, nibbles_2_to_4),
                false => self.jump_with_offset(Some(second_nibble), nibbles_3_to_4.into()),
            },
            0xC => self.random(second_nibble, nibbles_3_to_4),
            0xD => self.draw_to_screen(
                second_nibble as usize,
                third_nibble as usize,
                fourth_nibble as usize,
                renderer,
            ),
            0xE => match nibbles_3_to_4 {
                0x9E => self.skip_if_press_status(second_nibble, true),
                0xA1 => self.skip_if_press_status(second_nibble, false),
                _ => {}
            },
            0xF => match nibbles_3_to_4 {
                0x07 => self.set_register_to_delay_timer(second_nibble),
                0x15 => self.set_delay_timer_to_register(second_nibble),
                0x18 => todo!(),
                0x1E => todo!(),
                0x0A => todo!(),
                0x29 => todo!(),
                0x33 => todo!(),
                0x55 => todo!(),
                0x65 => todo!(),
                _ => {}
            },
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
        let mut y = self.registers[third_nibble] % 32;

        self.registers[15] = 0;

        'outer: for i in 0..fourth_nibble {
            let ith_byte = self.memory[self.index_register + i];
            let mut x = self.registers[second_nibble] % 64;
            'inner: for bit_value in 0..8 {
                // we subtract bit_value from 8 because bit_range_to_num works from right to left, so we need to flip it
                let sprite_bit = crate::bit_utils::bit_range_to_num(
                    ith_byte as u16,
                    8 - bit_value - 1,
                    8 - bit_value,
                )
                .unwrap();
                // we subtract y from SCREEN_HEIGHT because the pixels are indexed from bottom-left to top-right, so we flip vertically
                let pixel_index = ((crate::screen::SCREEN_HEIGHT - y as u32)
                    * crate::screen::SCREEN_WIDTH
                    + x as u32) as usize;
                let pixel = &mut renderer.instances[pixel_index];

                if sprite_bit == 1 {
                    if pixel.color.x > 0.1 {
                        pixel.color = cgmath::Vector4 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            w: 0.0,
                        };
                        self.registers[15] = 1;
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
        self.registers[register] = value;
    }

    fn add_to_register(&mut self, register: usize, addend: u8) {
        let register_to_change = &mut self.registers[register];
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

    fn stack_return(&mut self) {
        self.program_counter = *self.stack.top().unwrap() as usize;
        self.stack.pop();
    }

    fn call_subroutine(&mut self, address: usize) {
        self.stack.push(self.program_counter);
        self.program_counter = address;
    }

    fn skip_if_register_equals_value(&mut self, register: usize, value: u8) {
        if self.registers[register] == value {
            self.program_counter = self.program_counter + 2;
        }
    }

    fn skip_if_register_not_equal_to_value(&mut self, register: usize, value: u8) {
        if self.registers[register] != value {
            self.program_counter = self.program_counter + 2;
        }
    }

    fn skip_if_registers_equal(&mut self, register_x: usize, register_y: usize) {
        if self.registers[register_x] == self.registers[register_y] {
            self.program_counter = self.program_counter + 2;
        }
    }

    fn skip_if_registers_not_equal(&mut self, register_x: usize, register_y: usize) {
        if self.registers[register_x] != self.registers[register_y] {
            self.program_counter = self.program_counter + 2;
        }
    }

    fn set_register_to_other(&mut self, register_x: usize, register_y: usize) {
        self.registers[register_x] = self.registers[register_y];
    }

    /// Sets register X to the bitwise/binary logical disjunction (OR) of register X and register Y.
    fn binary_or(&mut self, register_x: usize, register_y: usize) {
        self.registers[register_x] = self.registers[register_x] | self.registers[register_y];
    }

    /// Sets register X to the bitwise/binary logical conjunction (AND) of register X and register Y.
    fn binary_and(&mut self, register_x: usize, register_y: usize) {
        self.registers[register_x] = self.registers[register_x] & self.registers[register_y];
    }

    /// Sets register X to the bitwise/binary logical exclusive OR (XOR) of register X and register Y.
    fn binary_xor(&mut self, register_x: usize, register_y: usize) {
        self.registers[register_x] = self.registers[register_x] ^ self.registers[register_y];
    }

    /// Sets register X to the sum of register X and register Y.
    /// Sets register F to 1 if result > 255, otherwise sets it to 0.
    fn add_registers(&mut self, register_x: usize, register_y: usize) {
        let temp: u16 = self.registers[register_x] as u16 + self.registers[register_y] as u16;
        if temp > std::u8::MAX.into() {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }
        self.registers[register_x] = (temp % (std::u8::MAX as u16 + 1)) as u8;
    }

    /// Subtracts right_register from left_register
    /// Sets register F to 1 if the left register is bigger than the right register otherwise 0.
    fn subtract_registers(&mut self, left_register: usize, right_register: usize) {
        let temp: i16 =
            self.registers[left_register] as i16 - self.registers[right_register] as i16;
        let carry: u8 = if temp < 0 { 0 } else { 1 };
        let borrowed: i16 = if carry == 0 { 1 } else { 0 };
        self.registers[15] = carry;
        self.registers[left_register] = (temp + ((std::u8::MAX as i16 + 1) * borrowed)) as u8;
    }

    /// Shifts register X by 1 bit to the right and sets register F to the bit shifted out.
    /// Sets register X to register Y first if the emulator has COSMAC VIP instructions.
    fn right_shift_on_register(&mut self, register_x: usize, register_y: usize) {
        self.registers[register_x] = if self.has_cosmac_vip_instructions {
            self.registers[register_y]
        } else {
            self.registers[register_x]
        };
        self.registers[15] = self.registers[register_x] & 1;
        self.registers[register_x] = self.registers[register_x] >> 1;
    }

    /// Shifts register X by 1 bit to the left and sets register F to the bit shifted out.
    /// Sets register X to register Y first if the emulator has COSMAC VIP instructions.
    fn left_shift_on_register(&mut self, register_x: usize, register_y: usize) {
        self.registers[register_x] = if self.has_cosmac_vip_instructions {
            self.registers[register_y]
        } else {
            self.registers[register_x]
        };
        self.registers[15] = (self.registers[register_x] & 0b10000000) >> 7;
        self.registers[register_x] = self.registers[register_x] << 1;
    }

    /// Sets the program counter to address + the value in register X (or register 0 if a COSMAC VIP).
    fn jump_with_offset(&mut self, register_x: Option<usize>, address: usize) {
        let register_to_use = match register_x {
            Some(register) => register,
            None => 0,
        };
        self.program_counter = address + self.registers[register_to_use] as usize;
    }

    /// Generates a rendom number and binary ANDs it with and_value.
    fn random(&mut self, register_x: usize, and_value: u8) {
        self.registers[register_x] = rand::random::<u8>() & and_value;
    }

    /// Skips one instruction if the key corresponding to the value in register X is equal to press_status
    fn skip_if_press_status(&mut self, register_x: usize, press_status: bool) {
        if *self
            .pressed
            .get(
                self.pressed_hex_map
                    .get(&(self.registers[register_x]))
                    .unwrap(),
            )
            .unwrap()
            == press_status
        {
            self.program_counter = self.program_counter + 2;
        }
    }

    /// Sets register X to the current value of the delay timer.
    fn set_register_to_delay_timer(&mut self, register_x: usize) {
        self.registers[register_x] = self.delay_timer.counter;
    }

    /// Sets the delay timer to the value in register X.
    fn set_delay_timer_to_register(&mut self, register_x: usize) {
        self.delay_timer.counter = self.registers[register_x];
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

#[cfg(test)]
mod emulator_tests {
    use super::Emulator;

    #[tokio::test]
    async fn test_jump() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.jump(0x210);
        assert_eq!(emulator.program_counter, 0x210);
    }

    #[tokio::test]
    async fn test_set_register() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.set_register(12, 41);
        assert_eq!(emulator.registers[12], 41);
    }

    #[tokio::test]
    async fn test_add_to_register() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.add_to_register(11, 15);
        assert_eq!(emulator.registers[11], 15);
        emulator.add_to_register(11, 215);
        assert_eq!(emulator.registers[11], 230);
    }

    #[tokio::test]
    async fn test_set_index_register() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.set_index_register(1411);
        assert_eq!(emulator.index_register, 1411);
    }

    #[tokio::test]
    async fn test_stack_return() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.stack.push(100);
        emulator.stack.push(200);
        emulator.stack_return();
        assert_eq!(emulator.program_counter, 200);
        assert_eq!(*emulator.stack.top().unwrap(), 100);
    }

    #[tokio::test]
    async fn test_call_subroutine() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.call_subroutine(400);
        assert_eq!(*emulator.stack.top().unwrap(), 200);
        assert_eq!(emulator.program_counter, 400);
    }

    #[tokio::test]
    async fn test_skip_if_register_equals_value() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.registers[0] = 1;
        emulator.skip_if_register_equals_value(0, 1);
        assert_eq!(emulator.program_counter, 202);
        emulator.skip_if_register_equals_value(0, 2);
        assert_eq!(emulator.program_counter, 202);
    }

    #[tokio::test]
    async fn test_skip_if_register_not_equal_to_value() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.registers[0] = 1;
        emulator.skip_if_register_not_equal_to_value(0, 2);
        assert_eq!(emulator.program_counter, 202);
        emulator.skip_if_register_not_equal_to_value(0, 1);
        assert_eq!(emulator.program_counter, 202);
    }

    #[tokio::test]
    async fn test_skip_if_registers_equal() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.registers[0] = 1;
        emulator.registers[1] = 1;
        emulator.skip_if_registers_equal(0, 1);
        assert_eq!(emulator.program_counter, 202);
        emulator.registers[0] = 2;
        emulator.skip_if_registers_equal(0, 1);
        assert_eq!(emulator.program_counter, 202);
    }

    #[tokio::test]
    async fn test_skip_if_registers_not_equal() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.registers[0] = 1;
        emulator.registers[1] = 2;
        emulator.skip_if_registers_not_equal(0, 1);
        assert_eq!(emulator.program_counter, 202);
        emulator.registers[0] = 2;
        emulator.skip_if_registers_not_equal(0, 1);
        assert_eq!(emulator.program_counter, 202);
    }

    #[tokio::test]
    async fn test_set_register_to_other() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 1;
        emulator.registers[1] = 15;
        emulator.set_register_to_other(0, 1);
        assert_eq!(emulator.registers[0], emulator.registers[1]);
    }

    #[tokio::test]
    async fn test_binary_or() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 0b1000;
        emulator.registers[1] = 0b0111;
        emulator.binary_or(0, 1);
        assert_eq!(emulator.registers[0], 0b1111);

        emulator.registers[2] = 0b1010;
        emulator.registers[3] = 0b0011;
        emulator.binary_or(2, 3);
        assert_eq!(emulator.registers[2], 0b1011);
    }

    #[tokio::test]
    async fn test_binary_and() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 0b1000;
        emulator.registers[1] = 0b1011;
        emulator.binary_and(0, 1);
        assert_eq!(emulator.registers[0], 0b1000);

        emulator.registers[3] = 0b1101;
        emulator.registers[4] = 0b1100;
        emulator.binary_and(3, 4);
        assert_eq!(emulator.registers[3], 0b1100);
    }

    #[tokio::test]
    async fn test_binary_xor() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 0b0110;
        emulator.registers[1] = 0b1101;
        emulator.binary_xor(0, 1);
        assert_eq!(emulator.registers[0], 0b1011);

        emulator.registers[4] = 0b1010;
        emulator.registers[5] = 0b0100;
        emulator.binary_xor(4, 5);
        assert_eq!(emulator.registers[4], 0b1110);
    }

    #[tokio::test]
    async fn test_add_registers() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 100;
        emulator.registers[1] = 50;
        emulator.add_registers(0, 1);
        assert_eq!(emulator.registers[0], 150);

        emulator.registers[13] = 200;
        emulator.registers[14] = 100;
        emulator.add_registers(13, 14);
        assert_eq!(emulator.registers[13], 44);
        assert_eq!(emulator.registers[15], 1);

        emulator.registers[3] = 200;
        emulator.registers[4] = 10;
        emulator.add_registers(3, 4);
        assert_eq!(emulator.registers[3], 210);
        assert_eq!(emulator.registers[15], 0);
    }

    #[tokio::test]
    async fn test_subtract_registers() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 100;
        emulator.registers[1] = 40;
        emulator.subtract_registers(0, 1);
        assert_eq!(emulator.registers[0], 60);
        assert_eq!(emulator.registers[15], 1);

        emulator.registers[3] = 40;
        emulator.registers[4] = 200;
        emulator.subtract_registers(3, 4);
        assert_eq!(emulator.registers[3], 96);
        assert_eq!(emulator.registers[15], 0);
    }

    #[tokio::test]
    async fn test_right_shift_on_register() {
        let mut emulator = Emulator::new(None, true).await;

        emulator.registers[0] = 0b100;
        emulator.registers[1] = 0b101;
        emulator.right_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b10);
        assert_eq!(emulator.registers[15], 1);

        emulator.registers[0] = 0b101;
        emulator.registers[1] = 0b1000;
        emulator.right_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b100);
        assert_eq!(emulator.registers[15], 0);

        emulator = Emulator::new(None, false).await;

        emulator.registers[0] = 0b1000;
        emulator.registers[1] = 0b110110;
        emulator.right_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b100);
        assert_eq!(emulator.registers[15], 0);

        emulator.registers[0] = 0b101;
        emulator.registers[1] = 0b110110;
        emulator.right_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b10);
        assert_eq!(emulator.registers[15], 1);
    }

    #[tokio::test]
    async fn test_left_shift_on_register() {
        let mut emulator = Emulator::new(None, true).await;

        emulator.registers[0] = 0b100;
        emulator.registers[1] = 0b101;
        emulator.left_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b1010);
        assert_eq!(emulator.registers[15], 0);

        emulator.registers[0] = 0b101;
        emulator.registers[1] = 0b10001101;
        emulator.left_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b00011010);
        assert_eq!(emulator.registers[15], 1);

        emulator = Emulator::new(None, false).await;

        emulator.registers[0] = 0b10001;
        emulator.registers[1] = 0b111010;
        emulator.left_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b100010);
        assert_eq!(emulator.registers[15], 0);

        emulator.registers[0] = 0b10001101;
        emulator.registers[1] = 0b10;
        emulator.left_shift_on_register(0, 1);
        assert_eq!(emulator.registers[0], 0b00011010);
        assert_eq!(emulator.registers[15], 1);
    }

    #[tokio::test]
    async fn test_jump_with_offset() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.registers[0] = 10;
        emulator.jump_with_offset(None, 100);
        assert_eq!(emulator.program_counter, 110);

        emulator = Emulator::new(None, false).await;
        emulator.program_counter = 200;
        emulator.registers[4] = 10;
        emulator.jump_with_offset(Some(4), 1000);
        assert_eq!(emulator.program_counter, 1010);
    }

    #[tokio::test]
    async fn test_skip_if_press_status() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 200;
        emulator.registers[0] = 0x2;
        *emulator.pressed.get_mut(&3).unwrap() = true;
        emulator.skip_if_press_status(0, true);
        assert_eq!(emulator.program_counter, 202);

        emulator.program_counter = 200;
        emulator.registers[1] = 0xE;
        *emulator.pressed.get_mut(&34).unwrap() = true;
        emulator.skip_if_press_status(1, true);
        assert_eq!(emulator.program_counter, 202);

        emulator.program_counter = 200;
        emulator.registers[2] = 0xF;
        *emulator.pressed.get_mut(&49).unwrap() = false;
        emulator.skip_if_press_status(2, false);
        assert_eq!(emulator.program_counter, 202);
    }

    #[tokio::test]
    async fn test_set_register_to_delay_timer() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.delay_timer.counter = 100;
        emulator.set_register_to_delay_timer(1);
        assert_eq!(emulator.registers[1], 100);
    }

    #[tokio::test]
    async fn test_set_delay_timer_to_register() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[1] = 100;
        emulator.set_delay_timer_to_register(1);
        assert_eq!(emulator.delay_timer.counter, 100);
    }
}
