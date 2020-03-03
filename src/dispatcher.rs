//#![feature(nll)]
use std::collections::VecDeque;
use std::sync::{Mutex, Arc, mpsc};
use std::thread;

use crate::commands;

#[derive(Clone, Copy, Debug)]
enum ThreadCommands {
    DIE,
    FINISH,
}

#[derive(Debug, Clone)]
pub struct JobResult {
    info: commands::JobInfo,
    results: Vec<String>,
}

pub struct Dispatcher {
    // A but fucky. but an arc or a mutex allows thread mutability from rust.
    // The send trait on the box below is for thread safety
    pub queue: Arc<Mutex<VecDeque<Box<dyn commands::Operation + Send>>>>,
    // The handle for our job dispatcher thread we will spawn
    thread_handle: Option<thread::JoinHandle<()>>,
    // Field for communication to/from the dispatcher, we can tell it to die here, and it will tell us if it is empty
    // TODO: Write a method for the dispatcher to check if it's empty instead.
    thread_command: Arc<Mutex<Option<ThreadCommands>>>,
    // The buffer for our threads to write results to
    results_queue: Arc<Mutex<VecDeque<JobResult>>>,
    // The number of concurrent jobs we'll run
    max_jobs: Arc<Mutex<usize>>,
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher { 
            //queue: Arc::new(Mutex::new(VecDeque::<commands::Operation>::new())),
            queue: Arc::new(Mutex::new(VecDeque::<Box<dyn commands::Operation + Send>>::new())),
            thread_handle: None,
            thread_command: Arc::new(Mutex::new(None)),
            results_queue: Arc::new(Mutex::new(VecDeque::<JobResult>::new())),
            max_jobs: Arc::new(Mutex::new(1)),
        }
    }
    
    // This is just a wrapper function to expose queue functionality
    // functionality is broken out so we can use the core logic inside our dispatcher thread
    // we specify static here so rust doesn't have to try to guess the lifetime, and send to allow access between threads
    pub fn push<T: commands::Operation + Send + 'static>(&self, item: T) {
        //queue_push(&self.queue.clone(), item);
        let boxeditem = Box::new(item);
        //queue_push(&self.queue.clone(), item);
        queue_push(&self.queue.clone(), boxeditem);
    }

    // This is just a wrapper function to expose queue functionality
    // functionality is broken out so we can use the core logic inside our dispatcher thread
    pub fn pop(&self) {
        queue_pop(&self.queue.clone());
    }

    // Check to see if our queue is empty
    pub fn is_empty(&self) -> bool {
        match queue_size(&self.queue.clone()) {
            0 => true,
            _ => false,
        }
    }

    // This is just a wrapper function to expose queue functionality
    // functionality is broken out so we can use the core logic inside our dispatcher thread
    pub fn queue_size(&self) -> usize {
        return queue_size(&self.queue.clone());
    }

    // Get the size of our results queue
    pub fn results_size(&self) -> usize {
        let results_queue = self.results_queue.lock().unwrap();
        return results_queue.len();
    }

    pub fn get_max_jobs(&self) -> usize {
        return get_max_jobs(&self.max_jobs.clone());
    }

    pub fn set_max_jobs(&self, new_max_jobs: usize) {
        set_max_jobs(new_max_jobs, &self.max_jobs.clone());
    }

    
    pub fn start(&mut self) {
        // Clone our arc and give it to the thread
        if self.thread_handle.is_some() {
            panic!("We have a thread handler for some reason, this shouldn't happen");
        }
       
        self.thread_handle = Some(self.spawn());
    }

    //block until our worker thread is stopped, consume dispatcher
    pub fn stop(&mut self) {

        if self.thread_handle.is_none() {
            panic!("Our thread handler is missing! We should have one at this point");
        }

        // Remove pending elements from the queue
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
        drop(queue);

        // Tell our thread to finish peacefully
        set_command(ThreadCommands::FINISH, &self.thread_command.clone());

        // Here we have to take the handle to avoid a copy of JoinHandle, as it doesn't exist
        let handle = self.thread_handle.take().unwrap();
        handle.join().expect("Problem waiting on our thread!");
    }

    // like stop, but without clearing the queue
    pub fn finish(&mut self) {
        if self.thread_handle.is_none() {
            panic!("Our thread handler is missing! We should have one at this point");
        }

        // Remove pending elements from the queue
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
        drop(queue);

        // Tell our thread to finish peacefully
        set_command(ThreadCommands::FINISH, &self.thread_command.clone());

        // Here we have to take the handle to avoid a copy of JoinHandle, as it doesn't exist
        let handle = self.thread_handle.take().unwrap();
        handle.join().expect("Problem waiting on our thread!");
    }

    // Consume self and kill all our currently executing children, MESSY
    pub fn kill(mut self) {

        if self.thread_handle.is_none() {
            panic!("Our thread handler is missing! We should have one at this point");
        }

        // Tell our thread to die and kill all running children
        println!("Setting command to die");
        set_command(ThreadCommands::DIE, &self.thread_command.clone());

        let handle = self.thread_handle.take().unwrap();
        println!("waiting on thread to die");
        handle.join().expect("Problem waiting on our thread!");

        println!("There may still be job windows open, close them.");
    }
    

    // TODO: this is our main thread spawning work loop. Make sure it's actually functioning
    fn spawn(&self) -> thread::JoinHandle<()> {
    
        // Clone our arcs to access our shared variables (these are mutexes)
        let queue_arc = self.queue.clone();
        let cmd_arc = self.thread_command.clone();
        let results_arc = self.results_queue.clone();
        let maxjobs_arc = self.max_jobs.clone();


        let handle = thread::spawn(move || {


            let mut job_vec: Vec<(std::process::Child, Box<dyn commands::Operation + Send>)> = Vec::new();

            loop {

                // process our commands, if any
                match get_command(&cmd_arc) {
                    Some(s) => {
                        match s {
                            ThreadCommands::FINISH => { // Finish execution when our queue is empty
                                println!("disp: we have been told to finish execution");
                                if job_vec.len() == 0 {
                                    break;
                                }
                            }
                            // TODO: This will not kill the started command windows. They'll need to be 
                            // exited manually for the time being
                            ThreadCommands::DIE => { // Kill our children and exit
                                println!("disp: we have been told die");
                                for job in job_vec.iter_mut() {
                                    job.0.kill().expect("disp: error killing job!");
                                }
                                break;
                            }
                        }
                    }
                    None => println!("disp: no command to process")
                }

                // First thing we do is see if any jobs have completed
                if !job_vec.is_empty() {
                    // Check over and over again until we have removed all completed children
                    loop {
                        match check_children(&mut job_vec) {
                            Some(s) => {
                                println!("disp: a job was returned: {:?}", s);
                                // Push our results onto our result queue
                                let mut results_queue = results_arc.lock().unwrap();
                                results_queue.push_front(s);
                                drop(results_queue);
                            },
                            None => {
                                println!("disp: all jobs were running");
                                break;
                            }
                        }
                    }
                }

                // Loop enough times to satisfy our max job limit
                for _ in 0..(open_jobs(&maxjobs_arc, &job_vec)) { // as isize to avoid negative wrap
                    
                    match queue_pop(&queue_arc) {
                        Some(mut x) => {
                            println!("disp: Popping a command off our queue");
                            // Execute the operation's preparation
                            let mut cmd = x.prepare();
                            // Spawn the command
                            let child = cmd.spawn().expect("disp: failed to async batch execute");
                            // push our info onto our vec
                            job_vec.push((child, x));

                        }
                        None => {
                            println!("disp: No more elements to pop");
                        }
                    }
                }
                

                // Delay each loop execution by 3 seconds
                std::thread::sleep(std::time::Duration::from_secs(10));
    
            }
            
        });

        return handle;

    }

}

// Loop through our running children and return a result on the first completed child one found, if one exists
fn check_children(job_vec: &mut Vec<(std::process::Child, Box<dyn commands::Operation + Send>)>) -> Option<JobResult> {

    // we ain't doin shit if there's a 0 size vec
    if job_vec.is_empty() {
        return None;
    }

    // TODO find a nice iterator method to go through this vec... I had trouble with the dyn trait with other methods
    for i in 0..job_vec.len() {
        println!("checker: Inspecting job: {}", i);

        // Check to see if our child is completed
        match job_vec[i].0.try_wait() {
            Ok(Some(status)) => {

                println!("checker: Child {} complete", job_vec[i].0.id());

                // Get our job's output
                let (info, files) = job_vec[i].1.getresults();
            
                // Run our job's cleanup routine
                job_vec[i].1.cleanup();

                //Communicate back to the main project a job success/failure
                println!("checker: Successfully got a status from a child!");
                println!("checker: chunk: {}", info.chunk_num);
                println!("checker: code: {}", status.code().unwrap() as usize);
                
                let result = JobResult {
                                info: info,
                                results: files,
                            };

                // remove the job from our list
                job_vec.remove(i);

                return Some(result); // git outta here, we can't delete any more if we just removed one
            }
            Ok(None) => println!("checker: Child is still running"),
            Err(e) => panic!("checker: Error attempting to wait: {}", e),
        }

    }

    // If we get here, we're still running every single job and there's nothing to return
    return None;

}

fn open_jobs(maxjobsarc: &Arc<Mutex<usize>>, runningjobs: &Vec<(std::process::Child, Box<dyn commands::Operation + Send>)>)
            -> isize {
    
    let maxjobs = maxjobsarc.lock().unwrap();
    return *maxjobs as isize - runningjobs.len() as isize;
}

fn thread_finish(command: &Arc<Mutex<Option<ThreadCommands>>>) -> bool {
    match get_command(command) {
        Some(s) => match s {
            ThreadCommands::FINISH => return true,
            _ => return false,
        }
        None => return false,
    }
}

fn get_max_jobs(maxjobs_arc: &Arc<Mutex<usize>>) -> usize {
    let max_jobs = maxjobs_arc.lock().unwrap();
    return *max_jobs;
}

fn set_max_jobs(new_maxjobs: usize, maxjobs_arc: &Arc<Mutex<usize>>) {
    let mut max_jobs = maxjobs_arc.lock().unwrap();
    *max_jobs = new_maxjobs;
}

// Use this when you want to clear the command as well as read it
fn take_command(command: &Arc<Mutex<Option<ThreadCommands>>>) -> Option<ThreadCommands> {
    let mut command = command.lock().unwrap();
    // Take the command, ensuring whatever it is now set to None, avoid duplicate messages
    return command.take();
}

// Use this when you want to check the value of the command instead of clearing it
fn get_command(command: &Arc<Mutex<Option<ThreadCommands>>>) -> Option<ThreadCommands> {
    let command = command.lock().unwrap();
    // Take the command, ensuring whatever it is now set to None, avoid duplicate messages
    return command.clone();
}

fn set_command(to_set: ThreadCommands, command: &Arc<Mutex<Option<ThreadCommands>>>) {
    let mut command = command.lock().unwrap();
    *command = Some(to_set);
}

fn queue_push<T>(queue: &Arc<Mutex<VecDeque<T>>>, item: T) {
    let mut queue = queue.lock().unwrap();
    queue.push_front(item);
}

fn queue_pop<T>(queue: &Arc<Mutex<VecDeque<T>>>) -> Option<T> {
    let mut queue = queue.lock().unwrap();
    return queue.pop_back();
}

fn queue_size<T>(queue: &Arc<Mutex<VecDeque<T>>>) -> usize {
    let queue = queue.lock().unwrap();
    return queue.len();
}