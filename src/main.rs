use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;

const MAX: usize = 10;

struct Buffer {
    inner: Mutex<BufferInner>,
    fill_cond: Condvar,
    empty_cond: Condvar,
}

impl Buffer {
    fn new() -> Self {
        Buffer {
            inner: Mutex::new(BufferInner {
                data: [Option::None; MAX],
                filled: 0,
                used: 0,
                count: 0,
            }),
            fill_cond: Condvar::new(),
            empty_cond: Condvar::new(),
        }
    }
}

struct BufferInner {
    data: [Option<i32>; MAX],
    filled: usize,
    used: usize,
    count: usize,
}

impl BufferInner {
    fn put(&mut self, value: i32) {
        self.data[self.filled] = Some(value);
        self.filled = (self.filled + 1) % MAX;
        self.count += 1;
    }

    fn get(&mut self) -> i32 {
        let tmp: Option<i32> = self.data[self.used];
        self.used = (self.used + 1) % MAX;
        self.count -= 1;
        tmp.unwrap()
    }
}

fn producer(buffer: &Buffer) {
    for i in 0..20 {
        let mut guard = buffer.inner.lock().unwrap();
        while guard.count == MAX {
            guard = buffer.empty_cond.wait(guard).unwrap();
        }

        guard.put(i);
        println!("producer: {}", i);
        buffer.fill_cond.notify_one();
    }
}

fn consumer(buffer: &Buffer) {
    for _ in 0..20 {
        let mut guard: MutexGuard<BufferInner> = buffer.inner.lock().unwrap();
        while guard.count == 0_usize {
            guard = buffer.fill_cond.wait(guard).unwrap();
        }

        let value = guard.get();
        println!("consumer: {}", value);
        buffer.empty_cond.notify_one();
    }
}

fn main() {
    let buffer = Arc::new(Buffer::new());
    let buffer1 = Arc::clone(&buffer);

    let p1 = thread::spawn(move || producer(&buffer));
    let c1 = thread::spawn(move || consumer(&buffer1));

    p1.join().unwrap();
    c1.join().unwrap();
}
