use std::sync::{Arc, Mutex};

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
