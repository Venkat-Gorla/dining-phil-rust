use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

pub struct ForkPool {
    forks: Vec<Arc<Mutex<()>>>,
}

impl ForkPool {
    pub fn new(num_forks: usize) -> Self {
        let forks = (0..num_forks)
            .map(|_| Arc::new(Mutex::new(())))
            .collect();

        Self { forks }
    }

    pub fn get_ordered_forks(&self, id: usize) -> (Arc<Mutex<()>>, Arc<Mutex<()>>) {
        let (left, right) = self.get_fork_pair(id);
        if id % 2 == 0 {
            (left, right)
        } else {
            (right, left)
        }
    }

    fn get_fork_pair(&self, index: usize) -> (Arc<Mutex<()>>, Arc<Mutex<()>>) {
        let left_fork = self.forks[index].clone();
        let right_fork = self.forks[(index + 1) % self.forks.len()].clone();
        (left_fork, right_fork)
    }
}

/// Simulates philosopher eating by acquiring two forks in deadlock-safe order
pub fn philosopher_eat(
    id: usize, 
    pool: &ForkPool, 
    count: usize, 
    tx: mpsc::Sender<String>
) {
    for _ in 0..count {
        println!("Philosopher {} is thinking", id);
        thread::sleep(Duration::from_millis(50));

        let (first, second) = pool.get_ordered_forks(id);
        let _lock1 = first.lock().unwrap();
        let _lock2 = second.lock().unwrap();

        println!("Philosopher {} is eating", id);
        tx.send(format!("Philosopher {} is eating", id)).unwrap();

        thread::sleep(Duration::from_millis(50));
    }
}

const NUM_PHILOSOPHERS: usize = 5;

pub fn run_philosophers() {
    const NUM_ITERATIONS: usize = 3;
    let pool = Arc::new(ForkPool::new(NUM_PHILOSOPHERS));
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for i in 0..NUM_PHILOSOPHERS {
        let pool_ref = Arc::clone(&pool);
        let tx_clone = tx.clone();
        let handle = thread::spawn(move || {
            philosopher_eat(i, &pool_ref, NUM_ITERATIONS, tx_clone);
        });
        handles.push(handle);
    }

    drop(tx); // Important: close channel after all senders are cloned

    for handle in handles {
        handle.join().unwrap();
    }

    let messages: Vec<_> = rx.iter().collect();
    assert_eq!(messages.len(), NUM_PHILOSOPHERS * NUM_ITERATIONS, 
               "Not all philosophers ate the expected number of times");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_pair_consistency() {
        assert!(NUM_PHILOSOPHERS >= 2, "Dining requires at least two philosophers");

        let pool = ForkPool::new(NUM_PHILOSOPHERS);
        for i in 0..NUM_PHILOSOPHERS {
            let (left, right) = pool.get_ordered_forks(i);
            assert!(
                !Arc::ptr_eq(&left, &right),
                "Left and right forks must not be the same for philosopher {i}"
            );
        }
    }

    #[test]
    fn test_run_philosophers_completes() {
        run_philosophers();
    }

    #[test]
    fn test_single_philosopher_eats() {
        use std::sync::mpsc;

        let pool = ForkPool::new(2);
        let (tx, rx) = mpsc::channel();

        const ITERATIONS: usize = 1;
        philosopher_eat(0, &pool, ITERATIONS, tx);

        let messages: Vec<_> = rx.iter().collect();
        assert_eq!(messages.len(), ITERATIONS);
        assert_eq!(messages[0], "Philosopher 0 is eating");
    }

    #[test]
    fn test_no_deadlock() {
        use std::sync::mpsc;
        use std::time::Duration;

        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            run_philosophers(); // simulate dining
            tx.send(()).unwrap(); // signal success
        });

        let timeout = Duration::from_secs(5);
        assert!(
            rx.recv_timeout(timeout).is_ok(),
            "Deadlock detected: philosophers did not finish in time"
        );
    }
}
