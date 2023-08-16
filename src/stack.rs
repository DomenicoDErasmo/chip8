pub struct Stack {
    contents: Vec<usize>,
}

impl Stack {
    pub fn new() -> Self {
        Self { contents: vec![] }
    }

    pub fn top(&self) -> Option<&usize> {
        self.contents.last()
    }

    pub fn push(&mut self, val: usize) {
        self.contents.push(val);
    }

    pub fn pop(&mut self) {
        self.contents.pop();
    }
}
