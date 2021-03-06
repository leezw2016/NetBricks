use std::default::Default;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};
use super::Executable;
/// A very simple round-robin scheduler. This should really be more of a DRR scheduler.
pub struct Scheduler {
    /// The set of runnable items. Note we currently don't have a blocked queue.
    run_q: Vec<Box<Executable>>,
    /// Next task to run.
    next_task: usize,
    sched_channel: Receiver<SchedulerCommand>,
}

pub enum SchedulerCommand {
    Add(Box<Executable + Send>),
    Run(Arc<Fn(&mut Scheduler) + Send + Sync>),
    Execute,
}

const DEFAULT_Q_SIZE: usize = 256;

impl Default for Scheduler {
    fn default() -> Scheduler {
        Scheduler::new()
    }
}

impl Scheduler {
    pub fn new() -> Scheduler {
        let (_, receiver) = sync_channel(0);
        Scheduler::new_with_channel(receiver)
    }

    pub fn new_with_channel(channel: Receiver<SchedulerCommand>) -> Scheduler {
        Scheduler::new_with_channel_and_capacity(channel, DEFAULT_Q_SIZE)
    }

    pub fn new_with_channel_and_capacity(channel: Receiver<SchedulerCommand>, capacity: usize) -> Scheduler {
        Scheduler {
            run_q: Vec::with_capacity(capacity),
            next_task: 0,
            sched_channel: channel,
        }
    }

    fn handle_request(&mut self, request: SchedulerCommand) {
        match request {
            SchedulerCommand::Add(ex) => self.run_q.push(ex),
            SchedulerCommand::Run(f) => f(self),
            SchedulerCommand::Execute => self.execute_loop(),
        }
    }

    pub fn handle_requests(&mut self) {
        while let Ok(cmd) = self.sched_channel.recv() {
            self.handle_request(cmd)
        }
        println!("Scheduler exiting");
    }

    /// Add a task to the current scheduler.
    pub fn add_task<T: Executable + 'static>(&mut self, task: T) {
        self.run_q.push(box task)
    }

    #[inline]
    fn execute_internal(&mut self) {
        {
            let task = &mut (&mut self.run_q[self.next_task]);
            task.execute()
        }
        let len = self.run_q.len();
        let next = self.next_task + 1;
        if next == len {
            self.next_task = 0;
            if let Ok(cmd) = self.sched_channel.try_recv() {
                self.handle_request(cmd);
            }
        } else {
            self.next_task = next;
        }
    }

    /// Run the scheduling loop.
    // TODO: Add a variable to stop the scheduler (for whatever reason).
    pub fn execute_loop(&mut self) {
        if !self.run_q.is_empty() {
            loop {
                self.execute_internal()
            }
        }
    }

    pub fn execute_one(&mut self) {
        if !self.run_q.is_empty() {
            self.execute_internal()
        }
    }
}
