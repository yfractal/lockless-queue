#[derive(Debug, PartialEq)]
pub enum QueueError {
    Full,
    Empty,
}

pub struct BlockQueue<T> {
    queue: Vec<Option<T>>,
    head: usize,
    tail: usize,
    count: usize,
    size: usize,
}

impl<T> BlockQueue<T> {
    pub fn new(size: usize) -> Self {
        let mut queue = Vec::with_capacity(size);
        queue.resize_with(size, || None);
        BlockQueue {
            queue,
            head: 0,
            tail: 0,
            count: 0,
            size,
        }
    }

    pub fn produce(&mut self, item: T) -> Result<(), QueueError> {
        if self.count == self.size {
            return Err(QueueError::Full);
        }
        self.queue[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.size;
        self.count += 1;
        Ok(())
    }

    pub fn consume(&mut self) -> Result<T, QueueError> {
        if self.count == 0 {
            return Err(QueueError::Empty);
        }
        let item = self.queue[self.head].take().unwrap();
        self.head = (self.head + 1) % self.size;
        self.count -= 1;
        Ok(item)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_produce_consume() {
        let mut queue = BlockQueue::new(2);
        assert!(queue.produce(1).is_ok());
        assert!(queue.produce(2).is_ok());
        assert_eq!(queue.produce(3), Err(QueueError::Full));
        assert_eq!(queue.consume().unwrap(), 1);
        assert!(queue.produce(3).is_ok());
        assert_eq!(queue.consume().unwrap(), 2);
        assert_eq!(queue.consume().unwrap(), 3);
        assert_eq!(queue.consume(), Err(QueueError::Empty));
    }

    #[test]
    fn test_is_empty() {
        let mut queue = BlockQueue::new(2);
        assert!(queue.is_empty());
        queue.produce(1).unwrap();
        assert!(!queue.is_empty());
        queue.consume().unwrap();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_is_full() {
        let mut queue = BlockQueue::new(2);
        assert!(!queue.is_full());
        queue.produce(1).unwrap();
        assert!(!queue.is_full());
        queue.produce(2).unwrap();
        assert!(queue.is_full());
    }
}
