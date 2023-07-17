pub const WIDTH: u32 = 4;
pub const HEIGHT: u32 = 2;

// TODO: tweak instance_displacement
pub const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> =
    cgmath::Vector3::new(WIDTH as f32 * 0.125, HEIGHT as f32 * 0.125, 0.0);

pub struct Pixel {
    pub vertices: [crate::vertex::Vertex; 4],
    pub indices: [u16; 6],
}

// TODO: get pixel to spawn in bottom left
impl Pixel {
    pub fn new(window: &winit::window::Window) -> Self {
        let width_half = window.inner_size().width as f32 / 200.0;
        let length_half = window.inner_size().height as f32 / 200.0;
        Self {
            vertices: [
                crate::vertex::Vertex {
                    position: [-0.5, -0.5, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0, -0.5, 0.0],
                    color: [0.0, 0.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0, -1.0, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-0.5, -1.0, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
            ],
            indices: [0, 1, 2, 0, 2, 3],
        }
    }
}
