use native;
use std::comm::channel;


struct WorkUnit {
    arg: int,
    callback: Sender<int>,
}


enum MessageToWorker {
    Work(WorkUnit),
    Halt,
}


struct WorkQueue {
    dispatcher: Sender<MessageToWorker>,
    worker_count: int,
}


impl WorkQueue {
    pub fn create(worker_count: int) -> WorkQueue {
        let (want_work, idle_worker) = channel::<Sender<MessageToWorker>>();
        let (dispatcher, dispatcher_inbox) = channel::<MessageToWorker>();

        for _ in range(0, worker_count) {
            // worker
            let want_work_copy = want_work.clone();
            native::task::spawn(proc() {
                let want_work = want_work_copy;
                loop {
                    let (reply_with_work, get_work_unit) = channel::<MessageToWorker>();
                    want_work.send(reply_with_work);
                    let work_unit = match get_work_unit.recv() {
                        Work(wu) => wu,
                        Halt     => return
                    };
                    let rv = work_unit.arg * 2;
                    work_unit.callback.send(rv);
                }
            });
        }

        // dispatcher
        native::task::spawn(proc() {
            let mut worker_count = worker_count;
            let inbox = dispatcher_inbox;
            let idle_worker = idle_worker;
            loop {
                match inbox.recv() {
                    Work(work_item) => {
                        idle_worker.recv().send(Work(work_item));
                    },
                    Halt => {
                        worker_count -= 1;
                        idle_worker.recv().send(Halt);
                        if worker_count == 0 {
                            return;
                        }
                    },
                };
            }
        });

        return WorkQueue{dispatcher: dispatcher, worker_count: worker_count};
    }


    pub fn execute(&self, arg: int) -> Receiver<int> {
        let (reply_with_rv, wait_for_rv) = channel::<int>();
        self.dispatcher.send(Work(WorkUnit{arg: arg, callback: reply_with_rv}));
        return wait_for_rv;
    }
}


impl Drop for WorkQueue {
    fn drop(&mut self) {
        for _ in range(0, self.worker_count) {
            self.dispatcher.send(Halt);
        }
    }
}


#[test]
fn test_queue() {
    let queue = WorkQueue::create(3);
    let mut promise_list: ~[Receiver<int>] = ~[];
    for c in range(0, 10) {
        let rv = queue.execute(c);
        promise_list.push(rv);
    }
    let return_list = promise_list.map(|promise| promise.recv());
    assert_eq!(return_list, ~[0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}
