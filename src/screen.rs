pub const SCREEN_WIDTH: u32 = 64;
pub const SCREEN_HEIGHT: u32 = 32;

pub struct Pixel {
    pub vertices: [crate::vertex::Vertex; 4],
    pub indices: [u16; 6],
}

// Setting all colors to 0.0 shows a black screen but we've made a bunch of pixels
impl Pixel {
    pub fn new() -> Self {
        let width = 2.0 / SCREEN_WIDTH as f32;
        let height = 2.0 / SCREEN_HEIGHT as f32;
        Self {
            vertices: [
                crate::vertex::Vertex {
                    position: [-1.0 + width, -1.0 + height, 0.0],
                    color: [0.0, 0.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0, -1.0 + height, 0.0],
                    color: [0.0, 0.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0, -1.0, 0.0],
                    color: [0.0, 0.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0 + width, -1.0, 0.0],
                    color: [0.0, 0.0, 0.0],
                },
            ],
            indices: [0, 1, 2, 0, 2, 3],
        }
    }
}
