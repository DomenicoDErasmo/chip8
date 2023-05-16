use std::collections::HashSet;
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};
use winit::event::ElementState;
use winit::window::Icon;
use winit::{event::KeyboardInput, platform::windows::WindowBuilderExtWindows};
use winit::{
    event::{Event, WindowEvent},
    platform::run_return::EventLoopExtRunReturn,
};
use winit::{event_loop::EventLoop, window::Window, window::WindowBuilder};
pub struct MyWindow {
    event_loop: EventLoop<()>,
    pub window: Window,
}

impl MyWindow {
    pub fn new() -> MyWindow {
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
        let window = MyWindow { event_loop, window };
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

    pub fn run(mut self) {
        let mut last_time = Self::current_timestamp();
        let mut counter = 0;
        const HERTZ: u32 = 60;

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

            // end of frame
            let mut current_time = Self::current_timestamp();

            let time_delta = match u32::try_from(current_time - last_time) {
                Ok(time) => time,
                Err(_) => panic!("Conversion error!"),
            };

            counter = counter + 1;
            if counter >= HERTZ {
                println!("A second has passed");
                counter = 0;
            }

            // end of processing for this visual frame
            sleep(Duration::new(
                0,
                Self::nanoseconds_per_cycle(HERTZ) - time_delta,
            ));

            current_time = Self::current_timestamp();
            last_time = current_time;
        }
    }

    fn current_timestamp() -> u128 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => time.as_millis(),
            Err(_) => panic!("System time before UNIX EPOCH!"),
        }
    }

    fn nanoseconds_per_cycle(frequency: u32) -> u32 {
        // taking duration of one cycle and multiplying by 1,000,000 to get the desired precision
        (1_000_000.0 / frequency as f32).round() as u32
    }
}
