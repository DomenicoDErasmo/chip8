pub mod stack {
    #[derive(Debug)]
    pub struct Stack<T> {
        stack: Vec<T>,
    }

    impl<T> Stack<T> {
        pub fn new() -> Stack<T> {
            Stack::<T> {
                stack: Vec::<T>::new()
            }
        }

        pub fn push(&mut self, value: T) {
            self.stack.push(value);
        }

        pub fn pop(&mut self) {
            self.stack.pop();
        }

        pub fn is_empty(&self) -> bool {
            self.stack.is_empty()
        }

        pub fn size(&self) -> usize {
            self.stack.len()
        }

        pub fn top(&self) -> Option<&T> {
            self.stack.last()
        }
    }

    #[cfg(test)]
    mod private_stack_tests {
        use crate::stack::stack::Stack;

        #[test]
        fn test_new() {
            let stack = Stack::<&str>::new();
            assert_eq!(stack.stack.capacity(), 0);
        }

        #[test]
        fn test_top() {
            let mut stack = Stack::<u16>::new();
            stack.stack.push(1);
            assert_eq!(*stack.top().unwrap(), 1);
        }
    }
}

#[cfg(test)]
mod stack_tests {
    use crate::stack::stack;

    #[test]
    fn test_push() {
        let mut stack = stack::Stack::<u16>::new();
        stack.push(4);
        assert_eq!(*stack.top().unwrap(), 4);
    }

    #[test]
    fn test_is_empty() {
        let mut stack = stack::Stack::<char>::new();
        assert!(stack.is_empty());
        stack.push('a');
        assert!(!stack.is_empty());
    }

    #[test]
    fn test_pop() {
        let mut stack = stack::Stack::<usize>::new();
        stack.push(3);
        assert!(!stack.is_empty());
        stack.pop();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_size() {
        let mut stack = stack::Stack::<String>::new();
        assert_eq!(stack.size(), 0);
        stack.push(String::from("hello"));
        assert_eq!(stack.size(), 1);
    }
}