pub mod window {
    use winit::event::{Event, WindowEvent};
    use winit::platform::windows::WindowBuilderExtWindows;
    use winit::window::Icon;
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

        // TODO: add sleep to normalize FPS (see emulate function in emulator mod)
        // TODO: add handling for key presses in desired keypad (see keyboard mod)
        pub fn run(self) {
            self.event_loop.run(move |event, _, control_flow| {
                control_flow.set_wait();
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == self.window.id() => control_flow.set_exit(),
                    Event::MainEventsCleared => {
                        self.window.request_redraw();
                    }
                    _ => (),
                }
            });
        }
    }
}
