pub struct Stack {
    contents: Vec<u16>,
}

impl Stack {
    pub fn new() -> Self {
        Self { contents: vec![] }
    }

    pub fn top(&self) -> Option<&u16> {
        self.contents.last()
    }

    pub fn push(&mut self, val: u16) {
        self.contents.push(val);
    }

    pub fn pop(&mut self) {
        self.contents.pop();
    }
}
