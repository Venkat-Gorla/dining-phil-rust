use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ForkPool {
    forks: Vec<Arc<Mutex<i32>>>,
}

impl ForkPool {
    pub fn new(num_forks: usize) -> Self {
        let forks = (0..num_forks)
            .map(|_| Arc::new(Mutex::new(0)))
            .collect();

        Self { forks }
    }

    pub fn get_fork_pair(&self, index: usize) -> (Arc<Mutex<i32>>, Arc<Mutex<i32>>) {
        let left_fork = self.forks[index].clone();
        let right_fork = self.forks[(index + 1) % self.forks.len()].clone();
        (left_fork, right_fork)
    }
}

/// Simulates philosopher eating by acquiring two forks in deadlock-safe order
pub fn philosopher_eat(id: usize, pool: &ForkPool, count: usize) {
    for _ in 0..count {
        println!("Philosopher {} is thinking", id);
        thread::sleep(Duration::from_millis(50));

        let (left, right) = pool.get_fork_pair(id);
        let (first, second) = if id % 2 == 0 {
            (left, right)
        } else {
            (right, left)
        };

        let _lock1 = first.lock().unwrap();
        let _lock2 = second.lock().unwrap();

        println!("Philosopher {} is eating", id);
        thread::sleep(Duration::from_millis(50));
    }
}

const NUM_PHILOSOPHERS: usize = 5;

pub fn run_philosophers() {
    let pool = Arc::new(ForkPool::new(NUM_PHILOSOPHERS));
    let mut handles = vec![];

    for i in 0..NUM_PHILOSOPHERS {
        let pool_ref = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            philosopher_eat(i, &pool_ref, 3);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
