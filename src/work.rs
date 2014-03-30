struct WorkUnit {
    arg: int,
    callback: Sender<int>,
}


struct WorkQueue {
    enqueue_work: Sender<WorkUnit>,
    worker_count: int,
}


impl WorkQueue {
    pub fn create(worker_count: int) -> WorkQueue {
        use native;
        use std::comm::channel;

        let (want_work, worker_wants_work) = channel::<Sender<WorkUnit>>();
        let (enqueue_work, have_new_job) = channel::<WorkUnit>();

        for _ in range(0, worker_count) {
            // worker
            let want_work_copy = want_work.clone();
            native::task::spawn(proc() {
                let want_work = want_work_copy;
                loop {
                    let (reply_with_work, get_work_unit) = channel::<WorkUnit>();
                    want_work.send(reply_with_work);
                    let work_unit = get_work_unit.recv();
                    if work_unit.arg == -1 {
                        return;
                    }
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
                let unit = have_new_job.recv();
                let worker = worker_wants_work.recv();
                if unit.arg == -1 {
                    worker_count -= 1;
                }
                worker.send(unit);
                if worker_count == 0 {
                    return;
                }
            }
        });

        return WorkQueue{enqueue_work: enqueue_work, worker_count: worker_count};
    }


    pub fn execute(&self, arg: int) -> Receiver<int> {
        let (reply_with_rv, wait_for_rv) = channel::<int>();
        self.enqueue_work.send(WorkUnit{arg: arg, callback: reply_with_rv});
        return wait_for_rv;
    }
}


impl Drop for WorkQueue {
    fn drop(&mut self) {
        for _ in range(0, self.worker_count) {
            let (reply_with_rv, _) = channel::<int>();
            self.enqueue_work.send(WorkUnit{arg: -1, callback: reply_with_rv});
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
