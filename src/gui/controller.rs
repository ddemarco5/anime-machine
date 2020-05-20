use cpp_core::{StaticUpcast, Ptr};
use qt_core::{QBox, QObject, slot, SlotNoArgs, SlotOfQString};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

use crate::gui::worker::{Worker};

pub struct Controller {
    // This is a dummy object because we need some thread to attach our slots to
    dummy_object: QBox<qt_core::QObject>,
    //pub workerthread: QBox<qt_core::QThread>,
    //workerthread: QBox<qt_core::QThread>,
    pub workerthreads: Arc<Mutex<Vec<QBox<qt_core::QThread>>>>,
    //pub workerthreads: QBox<qt_core::QThread>,
}

/*
impl StaticUpcast<QObject> for Controller {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.workerthread.as_ptr().static_upcast()
    }
}
*/

impl StaticUpcast<QObject> for Controller {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.dummy_object.as_ptr().static_upcast()
    }
}

impl Controller {

    pub fn new() -> Rc<Controller> {
        unsafe {
            let test = Controller {
                dummy_object: qt_core::QObject::new_0a(),
                //workerthreads: qt_core::QThread::new_0a(),
                workerthreads: Arc::new(Mutex::new(Vec::new())),
            };
            return Rc::new(test);
        }
    }

    // Create a new thread and attach a worker to it
    pub unsafe fn add_worker(&self, worker: &Worker) {
        
        let new_thread = qt_core::QThread::new_0a();
        // Take our worker we've been given and assign it to a controller thread.
        //worker.workerobj.move_to_thread(&self.workerthreads);
        worker.workerobj.move_to_thread(&new_thread);

        
        // Lock our thread vector
        let threadvec_arc = self.workerthreads.clone();
        let mut threadvec = threadvec_arc.lock().unwrap();
        // Push our new thread to our list
        threadvec.push(new_thread);
        //self.workerthreads.push(new_thread);
        
    }

    pub unsafe fn start_workers(&self) {
        println!("Starting threads");

        
        // Lock our thread vector
        let threadvec_arc = self.workerthreads.clone();
        let threadvec = threadvec_arc.lock().unwrap();

        for (i, thread) in threadvec.iter().enumerate() {
            println!("Started thread {}", i);
            thread.start_0a();
        }
        

        //self.workerthreads.start_0a();
    }

    pub unsafe fn terminate_workers(&self) {
        println!("Terminating threads");

        
        // Lock our thread vector
        let threadvec_arc = self.workerthreads.clone();
        let mut threadvec = threadvec_arc.lock().unwrap();

        for (i, thread) in threadvec.iter().enumerate() {
            println!("Terminating thread {}", i);
            thread.terminate();
        }

        threadvec.clear();
        
        //self.workerthreads.terminate();
    }

    #[slot(SlotNoArgs)]
    pub unsafe fn terminate_child(self: &Rc<Self>) {
        self.terminate_workers();
    }

}