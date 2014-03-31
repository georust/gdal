// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


//! write docs here

use std::comm::channel;

/// write docs here
pub type WorkUnit<ARG, RV> = (ARG, Sender<RV>);

/// write docs here
pub enum MessageToWorker<ARG, RV> {
    /// A work unit.
    Work(WorkUnit<ARG, RV>),
    /// Shut down the worker task.
    Halt,
}

/// write docs here
pub enum MessageToDispatcher<ARG, RV> {
    /// A work unit that needs to be dispatched to a worker task.
    Dispatch(WorkUnit<ARG, RV>),
    /// Halt all worker tasks.
    HaltAll,
    /// Register a new worker task.
    RegisterWorker(Sender<Sender<Sender<MessageToWorker<ARG, RV>>>>),
}

/// A queue that distributes work items to worker tasks.
pub struct WorkQueue<ARG, RV> {
    /// write docs here
    dispatcher: Sender<MessageToDispatcher<ARG, RV>>,
}

/// A proxy to a `WorkQueue`. It can be freely cloned to use from multiple tasks.
pub struct Dispatcher<ARG, RV> {
    /// write docs here
    inbox: Receiver<MessageToDispatcher<ARG, RV>>,
}

/// A proxy to a `WorkQueue`. It can be freely cloned to use from multiple tasks.
pub struct WorkQueueProxy<ARG, RV> {
    /// write docs here
    dispatcher: Sender<MessageToDispatcher<ARG, RV>>,
}

/// A worker that executes tasks from its parent queue.
pub struct Worker<ARG, RV> {
    priv ask_for_work: Sender<Sender<MessageToWorker<ARG, RV>>>,
}

/// Create a new work queue.
pub fn WorkQueue<ARG:Send, RV:Send>() -> (WorkQueue<ARG, RV>, Dispatcher<ARG, RV>) {
    let (dispatcher, dispatcher_inbox) = channel::<MessageToDispatcher<ARG, RV>>();
    return (
        WorkQueue{dispatcher: dispatcher},
        Dispatcher{inbox: dispatcher_inbox},
    );
}

impl<ARG:Send, RV:Send> WorkQueue<ARG, RV> {
    /// Create a copyable proxy that can be used to push work units.
    pub fn proxy(&self) -> WorkQueueProxy<ARG, RV> {
        return WorkQueueProxy{dispatcher: self.dispatcher.clone()};
    }

    /// Register a new worker. It will receive a sender where it can ask for tasks.
    pub fn register_worker(&self) -> Sender<Sender<MessageToWorker<ARG, RV>>> {
        let (reg_s, reg_r) = channel::<Sender<Sender<MessageToWorker<ARG, RV>>>>();
        self.dispatcher.send(RegisterWorker(reg_s));
        return reg_r.recv();
    }

    /// Create a new worker.
    pub fn worker(&self) -> Worker<ARG, RV> {
        return Worker{ask_for_work: self.register_worker()};
    }

    /// Push a work item to this queue.
    pub fn push(&self, arg: ARG) -> Receiver<RV> {
        let (rv, wait_for_rv) = channel::<RV>();
        self.dispatcher.send(Dispatch((arg, rv)));
        return wait_for_rv;
    }
}


// rustc complais "cannot implement a destructor on a structure with
// type parameters", but our destruction is safe, we only send
// a simple message on a channel.
#[unsafe_destructor]
impl<ARG:Send, RV:Send> Drop for WorkQueue<ARG, RV> {
    fn drop(&mut self) {
        self.dispatcher.send(HaltAll);
    }
}

impl<ARG:Send, RV:Send> Dispatcher<ARG, RV> {
    /// Run the dispatcher loop. It will stop when the parent WorkQueue
    /// object is dropped.
    pub fn run(&self) {
        let (want_work, idle_worker) = channel::<Sender<MessageToWorker<ARG, RV>>>();
        let mut worker_count = 0;
        let idle_worker = idle_worker;
        loop {
            match self.inbox.recv() {
                Dispatch(work_item) => {
                    idle_worker.recv().send(Work(work_item));
                },
                RegisterWorker(want_idle_sender) => {
                    worker_count += 1;
                    want_idle_sender.send(want_work.clone());
                }
                HaltAll => {
                    while worker_count > 0 {
                        idle_worker.recv().send(Halt);
                        worker_count -= 1;
                    }
                    return;
                },
            };
        }
    }
}

impl<ARG:Send, RV:Send> WorkQueueProxy<ARG, RV> {
    /// Push a work item to this queue.
    pub fn push(&self, arg: ARG) -> Receiver<RV> {
        let (rv, wait_for_rv) = channel::<RV>();
        self.dispatcher.send(Dispatch((arg, rv)));
        return wait_for_rv;
    }
}

impl<ARG:Send, RV:Send> Clone for WorkQueueProxy<ARG, RV> {
    fn clone(&self) -> WorkQueueProxy<ARG, RV> {
        return WorkQueueProxy{dispatcher: self.dispatcher.clone()};
    }
}

impl<ARG:Send, RV:Send> Worker<ARG, RV> {
    pub fn run(&self, fun: |arg: ARG| -> RV) {
        loop {
            let (idle, work_unit) = channel::<MessageToWorker<ARG, RV>>();
            self.ask_for_work.send(idle);
            match work_unit.recv() {
                Work((arg, rv)) => rv.send((fun)(arg)),
                Halt            => return
            };
        }
    }
}

#[cfg(test)]
mod test {
    use std::task::spawn;
    use super::WorkQueue;

    #[test]
    fn test_queue() {
        let (queue, dispatcher) = WorkQueue::<int, int>();
        spawn(proc() { dispatcher.run() });
        for _ in range(0, 3) {
            let worker = queue.worker();
            spawn(proc() { worker.run(|arg| arg * 2); });
        }

        let return_list: ~[int] =
            range(0, 10)
            .map(|c| queue.push(c))
            .map(|rv| rv.recv())
            .collect();

        assert_eq!(return_list, ~[0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    }

    #[test]
    fn test_enqueue_from_tasks() {
        let (queue, dispatcher) = WorkQueue::<int, int>();
        spawn(proc() { dispatcher.run() });
        for _ in range(0, 3) {
            let worker = queue.worker();
            spawn(proc() { worker.run(|arg| arg * 2); });
        }
        let mut promise_list: ~[Receiver<int>] = ~[];
        let queue_proxy = queue.proxy();
        for c in range(0, 10) {
            let queue_proxy_clone = queue_proxy.clone();
            let (done, promise) = channel::<int>();
            promise_list.push(promise);
            spawn(proc() {
                let done = done;
                let queue = queue_proxy_clone;
                let rv = queue.push(c);
                done.send(rv.recv());
            });
        }

        let return_list: ~[int] =
            promise_list
            .iter()
            .map(|promise| promise.recv())
            .collect();
        assert_eq!(return_list, ~[0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    }
}

#[cfg(test)]
mod bench {
    extern crate test;

    use self::test::BenchHarness;
    use super::WorkQueue;

    #[bench]
    fn bench_50_tasks_4_threads(b: &mut BenchHarness) {
        let (queue, dispatcher) = WorkQueue::<int, int>();
        spawn(proc() { dispatcher.run() });
        for _ in range(0, 4) {
            let worker = queue.worker();
            spawn(proc() { worker.run(|arg| arg * 2); });
        }
        b.iter(|| {
            let _: ~[int] =
                range(0, 50)
                .map(|_| queue.push(1))
                .map(|rv| rv.recv())
                .collect();
        });
    }

    #[bench]
    fn bench_spawn_5_workers(b: &mut BenchHarness) {
        b.iter(|| {
            let (queue, dispatcher) = WorkQueue::<int, int>();
            spawn(proc() { dispatcher.run() });
            for _ in range(0, 5) {
                let worker = queue.worker();
                spawn(proc() { worker.run(|arg| arg * 2); });
            }
        });
    }
}
