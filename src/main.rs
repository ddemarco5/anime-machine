use std::io::{self, Write};
// for thread shit
use std::thread;
use std::sync::{Mutex, Arc};
use std::env;

mod scene_parse;
mod commands;
mod project;
mod dispatcher;

fn main() {

    //env_test();   
    //kill_test();
    //work_test();
    //stop_test();
    //job_test();
    project_test();

}

fn env_test() {
    let cur_dir_path = env::current_dir().unwrap();
    let cur_dir = cur_dir_path.to_str().unwrap();

    println!("The current directory is: {}", cur_dir);

    let mut project = project::Project::new();

    project.set_base_path(String::from(cur_dir));

    project.save();

}

fn project_test() {

    let mut testproject = project::Project::from_file();

    if testproject.file_name.is_empty(){
        let parsed_scenes = scene_parse::build_records("ep1-vid-Scenes.csv").unwrap();
        println!("parsed {} lines from scene file.", parsed_scenes.len());


        println!("New project detected! Adding chunks from file and a filename");
        testproject.file_name = String::from("ep1-vid.mkv");
        for record in parsed_scenes {
            testproject.add_chunk(record);
        }
    }
    let mut dispatcher = dispatcher::Dispatcher::new();

    let targetchunk = &testproject.chunks[0];

    dispatcher.push(commands::make_split(&testproject, &targetchunk));

    dispatcher.start();

    while dispatcher.results_size() == 0 {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    let result = dispatcher.get_results();

    println!("Got result!: {:?}", result);


    dispatcher.finish();
    
}

/*
fn job_test() {
    let mut dispatcher = dispatcher::Dispatcher::new();

    // push commands
    for i in 0..4 {
        //dispatcher.push(commands::test_split());
        let tmpframe = (i+1) * 10;
        dispatcher.push(commands::test_split(i, (tmpframe-1), tmpframe));
    }

    dispatcher.start();

    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("Setting max jobs to 2");
    dispatcher.set_max_jobs(2);

    std::thread::sleep(std::time::Duration::from_secs(30));

    println!("Setting max jobs to 1");
    dispatcher.set_max_jobs(1);
    
    std::thread::sleep(std::time::Duration::from_secs(30));

    println!("Dispatcher's queue is empty, telling it to die.");
    dispatcher.finish();
}
*/

/*
fn stop_test() {
    let mut dispatcher = dispatcher::Dispatcher::new();

    // push commands
    for i in 0..4 {
        //dispatcher.push(commands::test_split());
        let tmpframe = (i+1) * 10;
        dispatcher.push(commands::test_split(i, (tmpframe-1), tmpframe));
    }

    dispatcher.start();

    std::thread::sleep(std::time::Duration::from_secs(10));

    dispatcher.stop();
    
}
*/

/*
fn kill_test() {

    let mut dispatcher = dispatcher::Dispatcher::new();

    // push commands
    for i in 0..4 {
        //dispatcher.push(commands::test_split());
        let tmpframe = (i+1) * 10;
        dispatcher.push(commands::test_split(i, (tmpframe-1), tmpframe));
    }

    dispatcher.start();

    std::thread::sleep(std::time::Duration::from_secs(10));
    
    println!("Killing the dispatcher!");
    dispatcher.kill();

}
*/

/*
fn work_test() {
    let mut dispatcher = dispatcher::Dispatcher::new();

    // push commands
    for i in 0..4 {
        //dispatcher.push(commands::test_split());
        let tmpframe = (i+1) * 10;
        dispatcher.push(commands::test_split(i, (tmpframe-1), tmpframe));
    }

    dispatcher.start();

    
    while !dispatcher.is_empty() {
        // sit in a loop and wait for our queue to be empty
        println!("Waiting for our queue to be empty - result size: {}", dispatcher.results_size());
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    
    
    /*

    println!("Adding 3 more commands");

    for _ in 0..3 {
        dispatcher.push(commands::test_encode());
    }
    
    // sit in a loop and wait for our queue to be empty
    while !dispatcher.is_empty() {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    */
    

    //println!("Running stop, waiting for the dispatcher thread to die");
    println!("Dispatcher's queue is empty, telling it to die.");
    dispatcher.stop();



    /*
    for _ in 0..2 {
        //let counter = Arc::clone(&counter);
        let handle = thread::spawn( || {
            dispatcher.push(8);
            /*
            println!("Hello I am a thread!");
            let mut num = counter.lock().unwrap();
            *num += 1;
            */
        });

        handles.push(handle);
    }

    for handle in &handles {
        let thread = handle.thread();
        println!("Got thread: {:?}", thread.id());
    }

    for handle in handles {
        handle.join().expect("error joining thread");
    }

    */



    /* LOGIC

    let mut testproject = project::Project::from_file();

    if testproject.file_name.is_empty(){
        let parsed_scenes = scene_parse::build_records("ep1-vid-Scenes.csv").unwrap();
        println!("parsed {} lines from scene file.", parsed_scenes.len());


        println!("New project detected! Adding chunks from file and a filename");
        testproject.file_name = String::from("ep1-vid.mkv");
        for record in parsed_scenes {
            testproject.add_chunk(record);
        }
    }

    let mut chunks_to_split: Vec<usize> = Vec::new();
    for i in 0..5 {
        chunks_to_split.push(i);
    }
    
    testproject.split_chunks(chunks_to_split);


    // save our project's state
    testproject.save();

    */

    println!("done!");
} */