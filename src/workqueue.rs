use native::task;
use std::comm::channel;


pub struct WorkUnit<ARG, RV> {
    arg: ARG,
    rv: Sender<RV>,
}


pub enum MessageToWorker<ARG, RV> {
    Work(WorkUnit<ARG, RV>),
    Halt,
}


enum MessageToDispatcher<ARG, RV> {
    Dispatch(WorkUnit<ARG, RV>),
    HaltAll,
    RegisterWorker(Sender<Sender<Sender<MessageToWorker<ARG, RV>>>>),
}


pub struct WorkQueue<ARG, RV> {
    dispatcher: Sender<MessageToDispatcher<ARG, RV>>,
}


pub struct WorkQueueProxy<ARG, RV> {
    dispatcher: Sender<MessageToDispatcher<ARG, RV>>,
}


impl<ARG:Send, RV:Send> WorkQueue<ARG, RV> {
    pub fn create() -> WorkQueue<ARG, RV> {
        let (dispatcher, dispatcher_inbox) = channel::<MessageToDispatcher<ARG, RV>>();

        // dispatcher
        task::spawn(proc() {
            let (want_work, idle_worker) = channel::<Sender<MessageToWorker<ARG, RV>>>();
            let mut worker_count = 0;
            let inbox = dispatcher_inbox;
            let idle_worker = idle_worker;
            loop {
                match inbox.recv() {
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
        });
        return WorkQueue{dispatcher: dispatcher};
    }

    pub fn proxy(&self) -> WorkQueueProxy<ARG, RV> {
        return WorkQueueProxy{dispatcher: self.dispatcher.clone()};
    }

    pub fn register_worker(&self) -> Sender<Sender<MessageToWorker<ARG, RV>>> {
        let (reg_s, reg_r) = channel::<Sender<Sender<MessageToWorker<ARG, RV>>>>();
        self.dispatcher.send(RegisterWorker(reg_s));
        return reg_r.recv();
    }

    pub fn push(&self, arg: ARG) -> Receiver<RV> {
        let (rv, wait_for_rv) = channel::<RV>();
        self.dispatcher.send(Dispatch(WorkUnit{arg: arg, rv: rv}));
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


impl<ARG:Send, RV:Send> WorkQueueProxy<ARG, RV> {
    pub fn push(&self, arg: ARG) -> Receiver<RV> {
        let (rv, wait_for_rv) = channel::<RV>();
        self.dispatcher.send(Dispatch(WorkUnit{arg: arg, rv: rv}));
        return wait_for_rv;
    }
}


impl<ARG:Send, RV:Send> Clone for WorkQueueProxy<ARG, RV> {
    fn clone(&self) -> WorkQueueProxy<ARG, RV> {
        return WorkQueueProxy{dispatcher: self.dispatcher.clone()};
    }
}


#[cfg(test)]
mod test {
    use native::task;
    use test::BenchHarness;
    use workqueue::{WorkQueue, MessageToWorker, Work};

    fn spawn_test_worker(queue: &WorkQueue<int, int>) {
        let want_work = queue.register_worker();
        task::spawn(proc() {
            let want_work = want_work;
            loop {
                let (idle, get_work_unit) = channel::<MessageToWorker<int, int>>();
                want_work.send(idle);
                let work_unit = match get_work_unit.recv() {
                    Work(wu) => wu,
                    Halt     => return
                };
                let rv = work_unit.arg * 2;
                work_unit.rv.send(rv);
            }
        });
    }


    #[test]
    fn test_queue() {
        let queue = WorkQueue::<int, int>::create();
        for _ in range(0, 3) {
            spawn_test_worker(&queue);
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
        let queue = WorkQueue::<int, int>::create();
        for _ in range(0, 3) {
            spawn_test_worker(&queue);
        }
        let mut promise_list: ~[Receiver<int>] = ~[];
        let queue_proxy = queue.proxy();
        for c in range(0, 10) {
            let queue_proxy_clone = queue_proxy.clone();
            let (done, promise) = channel::<int>();
            promise_list.push(promise);
            task::spawn(proc() {
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


    #[bench]
    fn bench_50_tasks_4_threads(b: &mut BenchHarness) {
        let queue = WorkQueue::<int, int>::create();
        for _ in range(0, 4) {
            spawn_test_worker(&queue);
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
            let queue = WorkQueue::<int, int>::create();
            for _ in range(0, 5) {
                spawn_test_worker(&queue);
            }
        });
    }
}
