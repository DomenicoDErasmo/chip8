pub const SCREEN_WIDTH: u32 = 64;
pub const SCREEN_HEIGHT: u32 = 32;
pub const PIXEL_WIDTH: f32 = 2.0 / SCREEN_WIDTH as f32;
pub const PIXEL_HEIGHT: f32 = 2.0 / SCREEN_HEIGHT as f32;

pub struct Pixel {
    pub vertices: [crate::vertex::Vertex; 4],
    pub indices: [u16; 6],
}

// Setting all colors to 0.0 shows a black screen but we've made a bunch of pixels
impl Pixel {
    pub fn new() -> Self {
        // we only want the rectangle to take up a portion of its cell to show a grid
        // remove when no longer debugging, because this causes weird lines on resize
        let grid_multiplier = 0.95;
        Self {
            vertices: [
                crate::vertex::Vertex {
                    position: [
                        -1.0 + PIXEL_WIDTH * grid_multiplier,
                        -1.0 + PIXEL_HEIGHT * grid_multiplier,
                        0.0,
                    ],
                },
                crate::vertex::Vertex {
                    position: [-1.0, -1.0 + PIXEL_HEIGHT * grid_multiplier, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0, -1.0, 0.0],
                },
                crate::vertex::Vertex {
                    position: [-1.0 + PIXEL_WIDTH * grid_multiplier, -1.0, 0.0],
                },
            ],
            indices: [0, 1, 2, 0, 2, 3],
        }
    }
}
