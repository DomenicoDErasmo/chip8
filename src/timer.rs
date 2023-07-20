pub struct Timer {
    counter: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn decrement(&mut self) {
        if self.counter > 0 {
            self.counter = self.counter - 1;
        }
    }
}
