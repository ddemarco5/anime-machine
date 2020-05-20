//#![windows_subsystem = "windows"]
use std::thread;
use std::sync::{Arc, Mutex};
use cpp_core::{CastInto, Ref, StaticUpcast, Ptr};
use qt_core::{QString, QBox, slot, SlotNoArgs, SlotOfQString, QObject, qs};
use qt_widgets::{QWidget, QLabel};
use std::rc::Rc;

use std::process::{Command, Stdio};
use std::io::BufRead;
use std::io::Read;
use std::os::windows::io::{FromRawHandle, AsRawHandle};

mod controller;
use controller::Controller;

mod worker;
use worker::Worker;

struct ThreadBox {
    widget: QBox<qt_widgets::QWidget>,
    controller: Rc<Controller>,
    worker: Rc<Worker>,
    textbox: QBox<qt_widgets::QTextEdit>,
    text_update_signal: QBox<qt_core::SignalOfQString>,
    start_button: QBox<qt_widgets::QPushButton>,
    stop_button: QBox<qt_widgets::QPushButton>,
    start_signal: QBox<qt_core::SignalNoArgs>,
}

impl StaticUpcast<QObject> for ThreadBox {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.widget.as_ptr().static_upcast()
    }
}

impl ThreadBox {
    fn new() -> Rc<ThreadBox> {
        unsafe {
            // Our window and layout
            let window = qt_widgets::QWidget::new_0a();
            window.resize_2a(480,360);
            //window.set_window_flag_1a(qt_core::WindowType::FramelessWindowHint);
            let layout = qt_widgets::QVBoxLayout::new_1a(&window);

            // Our text area
            let textbox = qt_widgets::QTextEdit::new();
            textbox.set_read_only(true);
            //textbox.set_text_background_color(&qt_gui::QColor::from_3_int(0,0,0));
            let palette = qt_gui::QPalette::new_copy(textbox.palette());
            palette.set_color_2a(qt_gui::q_palette::ColorRole::Base, &qt_gui::QColor::from_3_int(0,0,0));
            palette.set_color_2a(qt_gui::q_palette::ColorRole::Text, &qt_gui::QColor::from_3_int(255,255,255));
            textbox.set_palette(&palette);
            //textbox.set_text_color(&qt_gui::QColor::from_3_int(255,255,255));
            //textbox.set_plain_text(&qs("text"));
            layout.add_widget(&textbox);
            

            // Start Button
            let start_button = qt_widgets::QPushButton::from_q_string(&qs("Start"));
            layout.add_widget(&start_button);
            // Stop Button
            let stop_button = qt_widgets::QPushButton::from_q_string(&qs("Stop"));
            layout.add_widget(&stop_button);

            let controller = Controller::new();
            let worker = Worker::new();

            
            window.show();

            let object = Rc::new(
                ThreadBox {
                    widget: window,
                    controller: controller,
                    start_button: start_button,
                    stop_button: stop_button,
                    worker: worker,
                    start_signal: qt_core::SignalNoArgs::new(),
                    textbox: textbox,
                    text_update_signal: qt_core::SignalOfQString::new(),
                }
            );
            object.init();
            return object;
        }
    }

    unsafe fn init(self: &Rc<Self>) {
        // Attach button's clicked signal to the signal that will start the thread's action
        //self.start_button.clicked().connect(&self.startsignal);
        self.start_button.clicked().connect(&self.slot_start_command());
        
        // Connect the thread's action routine with the signal that will be emitted from the button
        self.start_signal.connect(&self.worker.slot_start_child());

        // Connect our text update signal to our text update function
        self.text_update_signal.connect(&self.slot_add_text());
        // Connect the worker's text update signal to our own text update signal
        self.worker.textsignal.connect(&self.text_update_signal);

        // Connect the stop button signal with the worker thread's terminate
        //self.stop_button.clicked().connect(self.controller.workerthread.slot_terminate());
        self.stop_button.clicked().connect(&self.slot_stop_command());
        
        // Move our worker to our thread controller
        //self.worker.workerobj.move_to_thread(&self.controller.workerthreads);
        self.controller.add_worker(&self.worker);
        //self.controller.add_worker(&self.worker);

        // Start our thread's execution
        //self.controller.workerthreads.start_0a();
        self.controller.start_workers();
    }

    
    #[slot(SlotOfQString)]
    unsafe fn add_text(self: &Rc<Self>, qstring: Ref<QString>) {
        self.textbox.move_cursor_1a(qt_gui::q_text_cursor::MoveOperation::End);
        self.textbox.insert_plain_text(qstring);
        self.textbox.move_cursor_1a(qt_gui::q_text_cursor::MoveOperation::End);
    }

    
    #[slot(SlotNoArgs)]
    unsafe fn start_command(self: &Rc<Self>) {
        let child_arc = self.worker.child.clone();
        let mut child = child_arc.lock().unwrap();
        match child.take() {
            Some(_) => {
                println!("Child is already running");
            },
            None => {
                self.textbox.clear();
                self.worker.textsignal.emit(&qs("Starting child\n"));
                self.start_signal.emit();
            }
        }   
    }

    
    #[slot(SlotNoArgs)]
    unsafe fn stop_command(self: &Rc<Self>) {
        println!("Stopping child");
        let child_arc = self.worker.child.clone();
        let mut child = child_arc.lock().unwrap();
        match child.take() {
            Some(mut x) => {
                let kill_result = x.kill().unwrap();
                println!("Stopped child with: {:?}", kill_result);
                self.worker.textsignal.emit(&qs(format!("Stopped child with: {:?}\n", kill_result)));
            },
            None => {
                println!("No thread to kill");
                self.worker.textsignal.emit(&qs("Nothing running\n"));
            }
        }   
    }
    
    /*
    #[slot(SlotNoArgs)]
    unsafe fn terminate_child(self: &Rc<Self>) {
        self.stop_command();
        //self.controller.workerthreads.terminate();
        self.controller.terminate_workers();
    }
    */

}


pub fn run() {

    qt_widgets::QApplication::init( |app| unsafe {

        let window = ThreadBox::new();

        // Make sure we stop our thread when we see that the program is about to quit
        //app.about_to_quit().connect(window.controller.workerthread.slot_terminate());
        //app.about_to_quit().connect(&window.slot_terminate_child());
        app.about_to_quit().connect(&window.controller.slot_terminate_child());

        qt_widgets::QApplication::exec()
    
    });
    
}