struct WorkUnit {
    arg: int,
    callback: Sender<int>,
}


enum MessageToWorker {
    Work(WorkUnit),
    Halt,
}


struct WorkQueue {
    enqueue_work: Sender<MessageToWorker>,
    worker_count: int,
}


impl WorkQueue {
    pub fn create(worker_count: int) -> WorkQueue {
        use native;
        use std::comm::channel;

        let (want_work, worker_wants_work) = channel::<Sender<MessageToWorker>>();
        let (enqueue_work, have_new_job) = channel::<MessageToWorker>();

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
            let have_new_job = have_new_job;
            let worker_wants_work = worker_wants_work;
            loop {
                let message_to_worker = have_new_job.recv();
                let worker = worker_wants_work.recv();
                match message_to_worker {
                    Halt => worker_count -= 1,
                    _    => {}
                }
                worker.send(message_to_worker);
                if worker_count == 0 {
                    return;
                }
            }
        });

        return WorkQueue{enqueue_work: enqueue_work, worker_count: worker_count};
    }


    pub fn execute(&self, arg: int) -> Receiver<int> {
        let (reply_with_rv, wait_for_rv) = channel::<int>();
        self.enqueue_work.send(Work(WorkUnit{arg: arg, callback: reply_with_rv}));
        return wait_for_rv;
    }
}


impl Drop for WorkQueue {
    fn drop(&mut self) {
        for _ in range(0, self.worker_count) {
            self.enqueue_work.send(Halt);
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
