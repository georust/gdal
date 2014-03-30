use native;
use std::comm::channel;


struct WorkUnit<ARG, RV> {
    arg: ARG,
    rv: Sender<RV>,
}


enum MessageToWorker<ARG, RV> {
    Work(WorkUnit<ARG, RV>),
    Halt,
}


enum MessageToDispatcher<ARG, RV> {
    Dispatch(WorkUnit<ARG, RV>),
    HaltAll,
    RegisterWorker(Sender<Sender<Sender<MessageToWorker<ARG, RV>>>>),
}


struct WorkQueue<ARG, RV> {
    dispatcher: Sender<MessageToDispatcher<ARG, RV>>,
}


impl<ARG:Send, RV:Send> WorkQueue<ARG, RV> {
    pub fn create() -> WorkQueue<ARG, RV> {
        let (dispatcher, dispatcher_inbox) = channel::<MessageToDispatcher<ARG, RV>>();

        // dispatcher
        native::task::spawn(proc() {
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

    pub fn register_worker(&self) -> Sender<Sender<MessageToWorker<ARG, RV>>> {
        let (reg_s, reg_r) = channel::<Sender<Sender<MessageToWorker<ARG, RV>>>>();
        self.dispatcher.send(RegisterWorker(reg_s));
        return reg_r.recv();
    }

    pub fn execute(&self, arg: ARG) -> Receiver<RV> {
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


#[test]
fn test_queue() {
    let queue = WorkQueue::<int, int>::create();
    for _ in range(0, 3) {
        let want_work = queue.register_worker();
        native::task::spawn(proc() {
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
    let mut promise_list: ~[Receiver<int>] = ~[];
    for c in range(0, 10) {
        let rv = queue.execute(c);
        promise_list.push(rv);
    }
    let return_list = promise_list.map(|promise| promise.recv());
    assert_eq!(return_list, ~[0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}
