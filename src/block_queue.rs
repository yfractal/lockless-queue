use std::sync::{Arc, RwLock};

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

pub struct Reader<T> {
    queue: Arc<RwLock<BlockQueue<T>>>,
}

impl<T> Reader<T> {
    pub fn new(queue: Arc<RwLock<BlockQueue<T>>>) -> Self {
        Reader { queue }
    }

    pub fn consume(&self) -> Result<T, QueueError> {
        let mut queue = self.queue.write().unwrap();
        queue.consume()
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.queue.read().unwrap();
        queue.is_empty()
    }
}
pub struct Writer<T> {
    queue: Arc<RwLock<BlockQueue<T>>>,
}

impl<T> Writer<T> {
    pub fn new(queue: Arc<RwLock<BlockQueue<T>>>) -> Self {
        Writer { queue }
    }

    pub fn produce(&self, item: T) -> Result<(), QueueError> {
        let mut queue = self.queue.write().unwrap();
        queue.produce(item)
    }

    pub fn is_full(&self) -> bool {
        let queue = self.queue.read().unwrap();
        queue.is_full()
    }
}

pub fn block_queue<T>(size: usize) -> (Reader<T>, Writer<T>) {
    let queue = Arc::new(RwLock::new(BlockQueue::<T>::new(size)));
    let reader = Reader::new(queue.clone());
    let writer = Writer::new(queue.clone());

    (reader, writer)
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

    #[test]
    fn test_in_threads() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let queue = Arc::new(Mutex::new(BlockQueue::new(2)));
        let queue1: Arc<Mutex<BlockQueue<i32>>> = queue.clone();
        let queue2 = queue.clone();

        let handle1 = thread::spawn(move || {
            let mut queue = queue1.lock().unwrap();
            queue.produce(1).unwrap();
            queue.produce(2).unwrap();
        });

        let handle2 = thread::spawn(move || {
            let mut queue = queue2.lock().unwrap();
            let mut c = 0;

            while c < 2 {
                if let Ok(item) = queue.consume() {
                    assert_eq!(item, c + 1);
                    c += 1;
                }
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
    }

    #[test]
    fn test_block_queue() {
        let (reader, writer) = block_queue(2);
        assert!(writer.produce(1).is_ok());
        assert!(reader.consume().is_ok());
    }

    #[test]
    fn test_block_queue_in_threads() {
        use std::thread;

        let (reader, writer) = block_queue(2);

        let handle1 = thread::spawn(move || {
            writer.produce(1).unwrap();
            writer.produce(2).unwrap();
        });

        let handle2 = thread::spawn(move || {
            let mut c = 0;

            while c < 2 {
                if let Ok(item) = reader.consume() {
                    assert_eq!(item, c + 1);
                    c += 1;
                }
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

    }

}
