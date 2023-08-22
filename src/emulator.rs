use rodio::Source;
use winit::event::ElementState::{Pressed, Released};

const FPS: f64 = 60.0;
const MEMORY_SIZE: usize = 4096;
const FONT_MEMORY_START: usize = 0x50;

pub struct Emulator {
    memory: [u8; MEMORY_SIZE],
    stack: crate::stack::Stack,
    delay_timer: crate::timer::Timer,
    sound_timer: crate::timer::Timer,
    pressed: std::collections::HashMap<u8, winit::event::ElementState>,
    pressed_hex_map: bimap::BiHashMap<u8, u8>,
    program_counter: usize,
    index_register: u16,
    registers: [u8; 16],
    has_cosmac_vip_instructions: bool,
}

impl Emulator {
    /// Builds the emulator.
    ///
    /// # Arguments:
    /// * `file_path`: An optional path to the ROM.
    /// * `has_cosmac_vip_instructions`: Determines whether some functions behave like they would on the COSMAC VIP.
    pub async fn new(file_path: Option<&str>, has_cosmac_vip_instructions: bool) -> Self {
        let mut memory = match file_path {
            Some(path) => Self::load_memory_from_rom(path),
            None => [0; MEMORY_SIZE],
        };
        Self::load_font(&mut memory);

        let stack = crate::stack::Stack::new();
        let delay_timer = crate::timer::Timer::new();
        let sound_timer = crate::timer::Timer::new();

        let program_counter = 0x200;
        let index_register = 0;

        let pressed = Self::load_pressed();
        let pressed_hex_map = Self::load_pressed_hex_map();

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

    /// Runs the ROM and consumes it upon completion (i.e. not passing self by reference).
    pub async fn run(mut self) {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new();
        let icon = Some(Self::load_icon(std::path::Path::new(
            "./resources/f_alear.png",
        )));

        let window: winit::window::Window = winit::window::WindowBuilder::new()
            .with_window_icon(icon)
            .build(&event_loop)
            .unwrap();

        let mut renderer = crate::renderer::RendererState::new(window).await;
        let timer_length = std::time::Duration::new(0, (1_000_000_000.0 / FPS) as u32);

        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        let file = std::fs::File::open("./resources/beep.mp3").unwrap();
        let source = rodio::Decoder::new(std::io::BufReader::new(file))
            .unwrap()
            .buffered();

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
                // Resize
                winit::event::WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                // Scale factor changed
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size);
                }
                // Pressed or released a key
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            state, scancode, ..
                        },
                    ..
                } => {
                    let casted_scancode = &(*scancode as u8);
                    if self.pressed.contains_key(casted_scancode) {
                        *self.pressed.get_mut(casted_scancode).unwrap() = *state;
                    }

                    // In get_key, we loop indefinitely until a key is pressed (or released in the COSMAC VIP)
                    let parsed_instructions = crate::instruction_format::InstructionFormat::new(
                        &self.get_instructions_from_memory(),
                    );
                    let trigger_state = if self.has_cosmac_vip_instructions {
                        Released
                    } else {
                        Pressed
                    };
                    // Check that the current instruction is get_key
                    if (*state == trigger_state)
                        & (parsed_instructions.first_nibble == 0xF)
                        & (parsed_instructions.nibbles_3_to_4 == 0x0A)
                    {
                        self.get_key(parsed_instructions.second_nibble, Some(*casted_scancode));
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
                // play sound if timer == 0
                if self.sound_timer.counter == 0 {
                    sink.append(source.clone());
                }

                // decrement timers
                self.delay_timer.decrement();
                self.sound_timer.decrement();

                // 12x a frame -> 720 / instructions per second on 60 FPS
                for _ in 0..12 {
                    // fetch
                    let instruction_bytes: [u8; 2] = self.get_instructions_from_memory();

                    // increment program counter for next instruction
                    self.program_counter = self.program_counter + 2;

                    // decode and execute
                    self.parse_instruction(&instruction_bytes, &mut renderer);
                }

                // render
                renderer.window().request_redraw();
            }
            _ => {}
        })
    }

    /// Loads an icon for the emulator from the specified path.
    fn load_icon(path: &std::path::Path) -> winit::window::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect("Failed to open icon")
    }

    /// Loads the current instructions from memory.
    fn get_instructions_from_memory(&mut self) -> [u8; 2] {
        self.memory[self.program_counter..self.program_counter + 2]
            .try_into()
            .expect("Expected to receive 2 values from memory")
    }

    /// Initializes the emulator's font.
    fn load_font(memory: &mut [u8; MEMORY_SIZE]) {
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
        memory[FONT_MEMORY_START..FONT_MEMORY_START + font.len()].clone_from_slice(font);
    }

    /// Initializes the pressed map
    fn load_pressed() -> std::collections::HashMap<u8, winit::event::ElementState> {
        std::collections::HashMap::from([
            // 1234
            (2, Released),
            (3, Released),
            (4, Released),
            (5, Released),
            // QWER
            (17, Released),
            (18, Released),
            (19, Released),
            (20, Released),
            // ASDF
            (31, Released),
            (32, Released),
            (33, Released),
            (34, Released),
            // ZXCV
            (46, Released),
            (47, Released),
            (48, Released),
            (49, Released),
        ])
    }

    /// Initializes the pressed scancode hex map
    ///
    /// COSMAC VIP Layout:
    /// 1 2 3 C
    /// 4 5 6 D
    /// 7 8 9 E
    /// A 0 B F
    ///
    /// QWERTY Keyboard Layout:
    /// 1 2 3 4
    /// Q W E R
    /// A S D F
    /// Z X C V
    ///
    /// Mapped to scancodes:
    ///
    ///  2  3  4  5
    /// 17 18 19 20
    /// 31 32 33 34
    /// 46 47 48 49
    fn load_pressed_hex_map() -> bimap::BiHashMap<u8, u8> {
        let mut pressed_hex_map = bimap::BiHashMap::new();
        pressed_hex_map.insert(0x0, 47);
        pressed_hex_map.insert(0x1, 2);
        pressed_hex_map.insert(0x2, 3);
        pressed_hex_map.insert(0x3, 4);
        pressed_hex_map.insert(0x4, 17);
        pressed_hex_map.insert(0x5, 18);
        pressed_hex_map.insert(0x6, 19);
        pressed_hex_map.insert(0x7, 31);
        pressed_hex_map.insert(0x8, 32);
        pressed_hex_map.insert(0x9, 33);
        pressed_hex_map.insert(0xA, 46);
        pressed_hex_map.insert(0xB, 48);
        pressed_hex_map.insert(0xC, 5);
        pressed_hex_map.insert(0xD, 20);
        pressed_hex_map.insert(0xE, 34);
        pressed_hex_map.insert(0xF, 49);
        pressed_hex_map
    }

    /// Executes an action based on the two provided actions.
    fn parse_instruction(
        &mut self,
        instruction: &[u8; 2],
        renderer: &mut crate::renderer::RendererState,
    ) {
        // Parse instructions
        let p = crate::instruction_format::InstructionFormat::new(instruction);

        // Choose an instruction to execute
        match p.first_nibble {
            0x0 => match p.fourth_nibble {
                0x0 => clear_screen(renderer),
                0xE => self.stack_return(),
                _ => {}
            },
            0x1 => self.jump(p.nibbles_2_to_4),
            0x2 => self.call_subroutine(p.nibbles_2_to_4),
            0x3 => self.skip_if_register_equals_value(p.second_nibble, p.nibbles_3_to_4),
            0x4 => self.skip_if_register_not_equal_to_value(p.second_nibble, p.nibbles_3_to_4),
            0x5 => self.skip_if_registers_equal(p.second_nibble, p.third_nibble),
            0x6 => self.set_register(p.second_nibble as usize, p.nibbles_3_to_4),
            0x7 => self.add_to_register(p.second_nibble as usize, p.nibbles_3_to_4),
            0x8 => match p.fourth_nibble {
                0x0 => self.set_register_to_other(p.second_nibble, p.third_nibble),
                0x1 => self.binary_or(p.second_nibble, p.third_nibble),
                0x2 => self.binary_and(p.second_nibble, p.third_nibble),
                0x3 => self.binary_xor(p.second_nibble, p.third_nibble),
                0x4 => self.add_registers(p.second_nibble, p.third_nibble),
                0x5 => self.subtract_registers(p.second_nibble, p.third_nibble),
                0x6 => self.right_shift_on_register(p.second_nibble, p.third_nibble),
                0x7 => self.subtract_registers(p.third_nibble, p.second_nibble),
                0xE => self.left_shift_on_register(p.second_nibble, p.third_nibble),
                _ => {}
            },
            0x9 => self.skip_if_registers_not_equal(p.second_nibble, p.third_nibble),
            0xA => self.set_index_register(p.nibbles_2_to_4),
            0xB => match self.has_cosmac_vip_instructions {
                true => self.jump_with_offset(None, p.nibbles_2_to_4),
                false => self.jump_with_offset(Some(p.second_nibble), p.nibbles_3_to_4.into()),
            },
            0xC => self.random(p.second_nibble, p.nibbles_3_to_4),
            0xD => self.draw_to_screen(
                p.second_nibble as usize,
                p.third_nibble as usize,
                p.fourth_nibble as usize,
                renderer,
            ),
            0xE => match p.nibbles_3_to_4 {
                0x9E => self.skip_if_press_status(p.second_nibble, Pressed),
                0xA1 => self.skip_if_press_status(p.second_nibble, Released),
                _ => {}
            },
            0xF => match p.nibbles_3_to_4 {
                0x07 => self.set_register_to_delay_timer(p.second_nibble),
                0x15 => self.set_delay_timer_to_register(p.second_nibble),
                0x18 => self.set_sound_timer_to_register(p.second_nibble),
                0x1E => self.add_to_index(p.second_nibble),
                0x0A => self.get_key(p.second_nibble, None),
                0x29 => self.set_index_register_to_font_character(p.second_nibble),
                0x33 => self.binary_coded_decimal_conversion(p.second_nibble),
                0x55 => self.store_registers_to_memory(p.second_nibble),
                0x65 => self.store_memory_to_registers(p.second_nibble),
                _ => {}
            },
            _ => {}
        }
    }

    /// Sets the program counter to the provided address.
    fn jump(&mut self, address: usize) {
        self.program_counter = address;
    }

    /// Draws "rows" number of rows of 8 pixels starting from the X and Y coordinates found in register X and register Y, respectively.
    fn draw_to_screen(
        &mut self,
        register_x: usize,
        register_y: usize,
        rows: usize,
        renderer: &mut crate::renderer::RendererState,
    ) {
        let mut y = self.registers[register_y] % 32;

        self.registers[15] = 0;

        'outer: for i in 0..rows {
            let ith_byte = self.memory[self.index_register as usize + i];
            let mut x = self.registers[register_x] % 64;
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
                        self.registers[0xF] = 1;
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

    /// Sets register X to the provided value.
    fn set_register(&mut self, register_x: usize, value: u8) {
        self.registers[register_x] = value;
    }

    /// Adds the provided addend to register X.
    fn add_to_register(&mut self, register_x: usize, addend: u8) {
        let register_to_change = &mut self.registers[register_x];
        let sum = *register_to_change + addend;
        *register_to_change = sum;
    }

    /// Sets the index register to the provided address.
    fn set_index_register(&mut self, address: usize) {
        self.index_register = address as u16;
    }

    /// Loads memory from the provided ROM.
    fn load_memory_from_rom(file_path: &str) -> [u8; MEMORY_SIZE] {
        let mut memory = [0; MEMORY_SIZE];

        let rom_contents = std::fs::read(file_path).unwrap();
        for (i, instruction) in rom_contents.iter().enumerate() {
            memory[0x200 + i] = *instruction;
        }

        memory
    }

    /// Sets the program coutner to the top of the stack and pops from the stack.
    fn stack_return(&mut self) {
        self.program_counter = *self.stack.top().unwrap() as usize;
        self.stack.pop();
    }

    /// Pushes the current program counter to the stack and sets the program counter to address.
    fn call_subroutine(&mut self, address: usize) {
        self.stack.push(self.program_counter);
        self.program_counter = address;
    }

    /// Skips the next instruction if register X equals the provided value.
    fn skip_if_register_equals_value(&mut self, register_x: usize, value: u8) {
        if self.registers[register_x] == value {
            self.program_counter = self.program_counter + 2;
        }
    }

    /// Skips the next instruction if register X does not equal the provided value.
    fn skip_if_register_not_equal_to_value(&mut self, register_x: usize, value: u8) {
        if self.registers[register_x] != value {
            self.program_counter = self.program_counter + 2;
        }
    }

    /// Skips the next instruction if registers X and Y are equal.
    fn skip_if_registers_equal(&mut self, register_x: usize, register_y: usize) {
        if self.registers[register_x] == self.registers[register_y] {
            self.program_counter = self.program_counter + 2;
        }
    }

    /// Skips the next instruction if registers X and Y are not equal.
    fn skip_if_registers_not_equal(&mut self, register_x: usize, register_y: usize) {
        if self.registers[register_x] != self.registers[register_y] {
            self.program_counter = self.program_counter + 2;
        }
    }

    /// Sets register X to register Y.
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
        let u8_max_cast = std::u8::MAX as u16;
        if temp > u8_max_cast {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[register_x] = (temp % (u8_max_cast + 1)) as u8;
    }

    /// Subtracts right_register from left_register
    /// Sets register F to 1 if the left register is bigger than the right register otherwise 0.
    fn subtract_registers(&mut self, left_register: usize, right_register: usize) {
        let temp: i16 =
            self.registers[left_register] as i16 - self.registers[right_register] as i16;
        let carry: u8 = if temp < 0 { 0 } else { 1 };
        let borrowed: i16 = if carry == 0 { 1 } else { 0 };
        self.registers[0xF] = carry;
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
        self.registers[0xF] = self.registers[register_x] & 1;
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
        self.registers[0xF] = (self.registers[register_x] & 0b10000000) >> 7;
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
    fn skip_if_press_status(
        &mut self,
        register_x: usize,
        press_status: winit::event::ElementState,
    ) {
        if *self
            .pressed
            .get(
                self.pressed_hex_map
                    .get_by_left(&(self.registers[register_x]))
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

    /// Sets the sound timer to the value in register X.
    fn set_sound_timer_to_register(&mut self, register_x: usize) {
        self.sound_timer.counter = self.registers[register_x];
    }

    /// Adds register X to the index register. Will set register F to 1 if the index register overflows.
    /// While this behavior is inconsistent between emulators, it appears that at least one game relies on
    /// setting register F to 1, whereas no games don't rely on it, so we opt for this standardized approach.
    fn add_to_index(&mut self, register_x: usize) {
        let temp: u32 = self.index_register as u32 + self.registers[register_x] as u32;
        let u16_max_cast = std::u16::MAX as u32;
        if temp > u16_max_cast {
            self.registers[0xF] = 1;
        }
        self.index_register = (temp % (u16_max_cast + 1)) as u16;
    }

    /// Blocks execution and waits for key input, but the timers should still be decreased while waiting.
    /// If a key is pressed while this instruction waits for input, the hex value will be put in VX and execution continues.
    /// If we reach this via the regular loop, then pass None. We also check the current program counter when we detect a new input.
    /// If the program counter would yield this function, pass in the scancode pressed.
    fn get_key(&mut self, register_x: usize, new_key_pressed: Option<u8>) {
        match new_key_pressed {
            Some(scancode) => {
                self.registers[register_x] = *self.pressed_hex_map.get_by_right(&scancode).unwrap();
            }
            None => self.program_counter = self.program_counter - 2,
        }
    }

    /// Sets the index register to the address of the hexidecimal character represented by the last nibble in register X.
    fn set_index_register_to_font_character(&mut self, register_x: usize) {
        let last_nibble =
            crate::bit_utils::bit_range_to_num(self.registers[register_x].into(), 0, 4).unwrap();
        self.index_register = FONT_MEMORY_START as u16 + 5 * last_nibble;
    }

    /// Takes the number in register X, converts it to three decimal digits, and stores the digits in memory at addresses starting with the index register.
    /// For example: 254 gets stored in memory[index_register..index_register+3] as [2, 5, 4]
    fn binary_coded_decimal_conversion(&mut self, register_x: usize) {
        let casted_index_register = self.index_register as usize;
        let converted_number = self.registers[register_x];

        let hundreds_place = converted_number / 100;
        let tens_place = converted_number / 10 - 10 * hundreds_place;
        let ones_place = converted_number - 100 * hundreds_place - 10 * tens_place;

        self.memory[casted_index_register] = hundreds_place;
        self.memory[casted_index_register + 1] = tens_place;
        self.memory[casted_index_register + 2] = ones_place;
    }

    /// Stores all register values from 0 to X to the memory starting at the index register.
    /// The COSMAC VIP incremented the index register while this function ran.
    fn store_registers_to_memory(&mut self, register_x: usize) {
        for register in 0..register_x + 1 {
            self.memory[self.index_register as usize + register] = self.registers[register];
        }
        if self.has_cosmac_vip_instructions {
            self.index_register = self.index_register + register_x as u16 + 1;
        }
    }

    /// Takes sequential values from memory starting at the index register and loads them to registers 0 to X.
    /// The COSCMAC VIP incremented the index register while this function ran.
    fn store_memory_to_registers(&mut self, register_x: usize) {
        for register in 0..register_x + 1 {
            self.registers[register] = self.memory[self.index_register as usize + register];
        }
        if self.has_cosmac_vip_instructions {
            self.index_register = self.index_register + register_x as u16 + 1;
        }
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
    use winit::event::ElementState::{Pressed, Released};

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
        *emulator.pressed.get_mut(&3).unwrap() = Pressed;
        emulator.skip_if_press_status(0, Pressed);
        assert_eq!(emulator.program_counter, 202);

        emulator.program_counter = 200;
        emulator.registers[1] = 0xE;
        *emulator.pressed.get_mut(&34).unwrap() = Pressed;
        emulator.skip_if_press_status(1, Pressed);
        assert_eq!(emulator.program_counter, 202);

        emulator.program_counter = 200;
        emulator.registers[2] = 0xF;
        *emulator.pressed.get_mut(&49).unwrap() = Released;
        emulator.skip_if_press_status(2, Released);
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

    #[tokio::test]
    async fn test_set_sound_timer_to_register() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[1] = 100;
        emulator.set_sound_timer_to_register(1);
        assert_eq!(emulator.sound_timer.counter, 100);
    }

    #[tokio::test]
    async fn test_add_to_index() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.index_register = 100;
        emulator.registers[1] = 21;
        emulator.add_to_index(1);
        assert_eq!(emulator.registers[0xF], 0);
        assert_eq!(emulator.index_register, 121);

        emulator.index_register = std::u16::MAX;
        emulator.registers[1] = 11;
        emulator.add_to_index(1);
        assert_eq!(emulator.registers[0xF], 1);
        assert_eq!(emulator.index_register, 10);
    }

    #[tokio::test]
    async fn test_get_key() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.program_counter = 102;
        emulator.get_key(1, None);
        assert_eq!(emulator.program_counter, 100);

        emulator.program_counter = 100;
        emulator.get_key(1, Some(3));
        assert_eq!(
            emulator.registers[1],
            *emulator.pressed_hex_map.get_by_right(&3).unwrap()
        );
    }

    #[tokio::test]
    async fn test_set_index_register_to_font_character() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 0xAB;
        emulator.set_index_register_to_font_character(0);
        let casted_index_register = emulator.index_register as usize;
        assert_eq!(
            &emulator.memory[casted_index_register..casted_index_register + 5],
            &[0xE0, 0x90, 0xE0, 0x90, 0xE0]
        );

        emulator.registers[0] = 0xA9;
        emulator.set_index_register_to_font_character(0);
        assert_eq!(
            &emulator.memory[casted_index_register..casted_index_register + 5],
            &[0xF0, 0x90, 0xF0, 0x10, 0xF0]
        );
        assert_ne!(
            &emulator.memory[casted_index_register..casted_index_register + 5],
            &[0xF0, 0x90, 0xF0, 0x10, 0xF1]
        );
    }

    #[tokio::test]
    async fn test_binary_coded_decimal_conversion() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.index_register = 0x300;
        emulator.registers[0] = 156;
        emulator.binary_coded_decimal_conversion(0);

        let casted_index_register = emulator.index_register as usize;
        assert_eq!(
            &emulator.memory[casted_index_register..casted_index_register + 3],
            &[1, 5, 6]
        );
    }

    #[tokio::test]
    async fn test_store_registers_to_memory() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.registers[0] = 4;
        emulator.registers[1] = 2;
        emulator.index_register = 0x200;
        emulator.store_registers_to_memory(1);
        assert_eq!(emulator.memory[0x200], 4);
        assert_eq!(emulator.memory[0x201], 2);
        assert_eq!(emulator.index_register, 0x202);

        emulator = Emulator::new(None, false).await;
        emulator.registers[0] = 4;
        emulator.registers[1] = 2;
        emulator.index_register = 0x200;
        emulator.store_registers_to_memory(1);
        assert_eq!(emulator.memory[0x200], 4);
        assert_eq!(emulator.memory[0x201], 2);
        assert_eq!(emulator.index_register, 0x200);
    }

    #[tokio::test]
    async fn test_store_memory_to_registers() {
        let mut emulator = Emulator::new(None, true).await;
        emulator.memory[0x200] = 4;
        emulator.memory[0x201] = 2;
        emulator.index_register = 0x200;
        emulator.store_memory_to_registers(1);
        assert_eq!(emulator.registers[0], 4);
        assert_eq!(emulator.registers[1], 2);
        assert_eq!(emulator.index_register, 0x202);

        emulator = Emulator::new(None, false).await;
        emulator.memory[0x200] = 4;
        emulator.memory[0x201] = 2;
        emulator.index_register = 0x200;
        emulator.store_memory_to_registers(1);
        assert_eq!(emulator.registers[0], 4);
        assert_eq!(emulator.registers[1], 2);
        assert_eq!(emulator.index_register, 0x200);
    }
}
