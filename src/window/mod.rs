use crate::emulator;
use spin_sleep::SpinSleeper;
use std::collections::HashSet;
use std::num::IntErrorKind;
use std::time::{Duration, SystemTime};
use winit::event::ElementState;
use winit::window::Icon;
use winit::{event::KeyboardInput, platform::windows::WindowBuilderExtWindows};
use winit::{
    event::{Event, WindowEvent},
    platform::run_return::EventLoopExtRunReturn,
};
use winit::{event_loop::EventLoop, window::Window, window::WindowBuilder};

pub struct MyWindow {
    _emulator: emulator::Emulator,
    event_loop: EventLoop<()>,
    pub window: Window,
}

impl MyWindow {
    pub fn new(emulator: emulator::Emulator) -> MyWindow {
        let event_loop = EventLoop::new();

        let window = match WindowBuilder::new()
            .with_title("CHIP-8 Emulator")
            .with_window_icon(Self::build_icon())
            .with_taskbar_icon(Self::build_icon())
            .build(&event_loop)
        {
            Ok(window) => window,
            Err(_) => panic!("Couldn't make window"),
        };
        let window = MyWindow {
            _emulator: emulator,
            event_loop,
            window,
        };
        window
    }

    fn build_icon() -> Option<Icon> {
        let image = image::open("resources/f_alear.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();

        let icon = match Icon::from_rgba(rgba, width, height) {
            Ok(icon) => Some(icon),
            Err(_) => None,
        };
        icon
    }

    pub fn run(&mut self) -> Result<(), IntErrorKind> {
        let mut last_time = Self::current_timestamp();
        let mut frame_counter = 0;
        const FRAMES_PER_SECOND: u32 = 60;

        let mut running = true;
        let killswitch_scancode = 25;

        let valid_keys = HashSet::<u32>::from([
            2,
            3,
            4,
            5,
            16,
            17,
            18,
            19,
            30,
            31,
            32,
            33,
            44,
            45,
            46,
            47,
            killswitch_scancode.clone(),
        ]);

        // native Windows sleep is inaccurate so we use a crate
        let spin_sleeper = SpinSleeper::new(100_000);

        while running {
            // process event
            self.event_loop.run_return(|event, _, control_flow| {
                control_flow.set_wait();

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => running = false,
                    Event::MainEventsCleared => control_flow.set_exit(),
                    Event::WindowEvent {
                        event:
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        scancode,
                                        virtual_keycode: Some(key),
                                        ..
                                    },
                                ..
                            },
                        ..
                    } => match scancode == killswitch_scancode {
                        true => running = false,
                        false => match valid_keys.contains(&scancode) {
                            true => println!("{} pressed - key {:#?}", scancode, key),
                            false => {}
                        },
                    },
                    _ => (),
                }
            });

            // fetch, decode, execute
            const INSTRUCTIONS_PER_SECOND: u32 = 720;
            const INSTRUCTIONS_PER_FRAME: u32 = INSTRUCTIONS_PER_SECOND / FRAMES_PER_SECOND;

            assert!(INSTRUCTIONS_PER_SECOND > FRAMES_PER_SECOND);

            for _ in 0..=INSTRUCTIONS_PER_FRAME {
                // fetch, decode, execute
                // let instruction = self.emulator.fetch();
                // let parsed_instruction = emulator::decode(instruction)?;
                // self.emulator.increment_program_counter();
                // parsed_instruction.execute(&mut self.emulator);
            }

            // end of frame
            let mut current_time = Self::current_timestamp();

            let time_delta = u32::try_from(current_time - last_time).unwrap_or(0);

            frame_counter = frame_counter + 1;
            if frame_counter >= FRAMES_PER_SECOND {
                println!("A second has passed");
                frame_counter = 0;
            }

            // end of processing for this visual frame
            spin_sleeper.sleep(Duration::new(
                0,
                Self::nanoseconds_per_cycle(FRAMES_PER_SECOND) - time_delta,
            ));

            current_time = Self::current_timestamp();
            last_time = current_time;
        }
        Ok(())
    }

    fn current_timestamp() -> u128 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => time.as_nanos(),
            Err(_) => panic!("System time before UNIX EPOCH!"),
        }
    }

    fn nanoseconds_per_cycle(frequency: u32) -> u32 {
        (1_000_000_000.0 / frequency as f32).round() as u32
    }
}
