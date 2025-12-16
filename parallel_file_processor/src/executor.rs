use std::{
    sync::{Arc, Mutex, Condvar},
    thread::{self, JoinHandle},
    collections::VecDeque,
    // Note: Duration is only used for the Condvar timeout logic in Worker loop, not for async.
    time::Duration, 
};

// Type alias for the job to be executed.
pub type WorkTask = Box<dyn FnOnce() + Send + 'static>;

pub struct TaskExecutor {
    workers: Vec<Worker>,
    job_queue: Arc<Mutex<VecDeque<WorkTask>>>, 
    wakeup_signal: Arc<Condvar>,
    stop_signal: Arc<Mutex<bool>>, 
}

impl TaskExecutor {
    pub fn new(capacity: usize) -> TaskExecutor {
        assert!(capacity > 0);

        // Core concurrency components from the standard library
        let job_queue = Arc::new(Mutex::new(VecDeque::new()));
        let wakeup_signal = Arc::new(Condvar::new());
        let stop_signal = Arc::new(Mutex::new(false));

        let mut workers = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            workers.push(Worker::start(
                Arc::clone(&job_queue),
                Arc::clone(&wakeup_signal),
                Arc::clone(&stop_signal),
            ));
        }

        TaskExecutor {
            workers,
            job_queue,
            wakeup_signal,
            stop_signal,
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Check termination flag before accepting work
        if *self.stop_signal.lock().unwrap() {
            eprintln!("Worker pool is shutting down; task rejected.");
            return;
        }
        
        let work = Box::new(f);
        self.job_queue.lock().unwrap().push_back(work);
        self.wakeup_signal.notify_one(); // Wake up one waiting worker
    }
}

impl Drop for TaskExecutor {
    fn drop(&mut self) {
        // 1. Signal workers to stop
        {
            let mut flag = self.stop_signal.lock().unwrap();
            *flag = true;
        }

        // 2. Wake all workers so they can see the stop signal
        self.wakeup_signal.notify_all();

        // 3. Wait for all workers to finish (join handles)
        for worker in &mut self.workers {
            if let Some(handle) = worker.thread_handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread_handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn start(
        queue: Arc<Mutex<VecDeque<WorkTask>>>,
        notifier: Arc<Condvar>,
        termination_flag: Arc<Mutex<bool>>,
    ) -> Worker {
        let thread_handle = thread::spawn(move || loop { // std::thread::spawn
            let mut queue_guard = queue.lock().unwrap();
            
            // Block worker until a task arrives or the pool is shutting down
            let guard = notifier
                .wait_while(queue_guard, |q| q.is_empty() && !*termination_flag.lock().unwrap())
                .unwrap();
            queue_guard = guard;

            // Check termination signal after waking up
            if *termination_flag.lock().unwrap() {
                break;
            }
            
            if let Some(work) = queue_guard.pop_front() {
                drop(queue_guard); // Release Mutex lock immediately
                work(); // Execute the task
            } else {
                 thread::yield_now(); 
            }
        });

        Worker {
            thread_handle: Some(thread_handle),
        }
    }
}