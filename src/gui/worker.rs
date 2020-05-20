
use std::sync::{Arc, Mutex};
use cpp_core::{StaticUpcast, Ptr};
use qt_core::{QBox, slot, SlotNoArgs, QObject};
use std::rc::Rc;

use std::process::{Command, Stdio};
use std::io::Read;

pub struct Worker {
    pub workerobj: QBox<QObject>,
    pub textsignal: QBox<qt_core::SignalOfQString>,
    pub child: Arc<Mutex<Option<std::process::Child>>>
}

impl StaticUpcast<QObject> for Worker {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.workerobj.as_ptr().static_upcast()
    }
}

impl Worker {

    pub fn new() -> Rc<Worker> {
        unsafe {
            Rc::new(Worker {
                workerobj: QObject::new_0a(),
                textsignal: qt_core::SignalOfQString::new(),
                child: Arc::new(Mutex::new(None))
            })
        }
    }

    #[slot(SlotNoArgs)]
    pub unsafe fn start_child(self: &Rc<Self>) {
        println!("hello from thread!");
        
        //let child_arc = self.child.clone();
        let child_arc = &self.child.clone();
        let mut child = child_arc.lock().unwrap();

        let mut local_child = Command::new("ping")
                                        .arg("-n")
                                        .arg("50")
                                        .arg("google.com")
                                        .stdin(Stdio::null())
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::piped())
                                        .spawn()
                                        .expect("Failed to start command");
        

        //let childreader = std::io::BufReader::new(self.child.stdout.take().unwrap());
        let childreader = std::io::BufReader::new(local_child.stdout.take().unwrap());
        let mut byte_iterator = childreader.bytes();

        *child = Some(local_child);

        // Drop the child from this scope to free the lock
        drop(child);

        loop {
            //println!("loop iteration");
            //std::thread::sleep(std::time::Duration::from_millis(10));

            // Read from the bytes iterator
            match byte_iterator.next() {
                Some(x) => {
                    match x {
                        Ok(x) => {
                            if x != 0xD { // This is unicode for carriage return, it's not 1950, so we can skip it
                                //println!("got a byte: {:X?},{}", x, x as char);
                                let char_slice = [x];
                                let str = std::str::from_utf8(&char_slice).expect("Failed to convert stdout to str");

                                // I'm not sure if emitting a signal for every character is the best way to go about this, 
                                // but it's the easiest way for now, since this read is a blocking operation
                                self.textsignal.emit(&qt_core::QString::from_std_str(str));
                            }
                        }
                        Err(e) => {
                            println!("got an error: {}", e)
                        }
                    }
                },
                None => {
                    println!("got EOF");
                    break;
                },
            }
        }
        // We're done. We need to make sure that the child is gone
        let mut child = child_arc.lock().unwrap();
        *child = None;
        drop(child);

        println!("goodbye from thread!");
    }

}