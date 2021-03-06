use std::io::{self, Write};
// for thread shit
use std::thread;
use std::sync::{Mutex, Arc};
use std::env;

mod scene_parse;
mod commands;
mod project;
mod dispatcher;
mod logic;
mod fileinfo;
mod gui;

fn main() {

    //env_test();   
    //kill_test();
    //work_test();
    //stop_test();
    //job_test();
    //project_test();

    gui::run();
    

    //let machine = logic::Machine::new(String::from("ep1-vid.mkv"));
    //machine.start(3);

    //let test = project::Project::open("ep1-vid.mkv".to_string());

    //println!("{:?}", test);

    //test.save();

}

/*
fn env_test() {
    let cur_dir_path = env::current_dir().unwrap();
    let cur_dir = cur_dir_path.to_str().unwrap();

    println!("The current directory is: {}", cur_dir);

    let mut project = project::Project::new();

    project.set_base_path(String::from(cur_dir));

    project.save();

}
*/

/*
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
    dispatcher.start();

    // push our split command
    dispatcher.push(commands::make_split(&testproject, testproject.get_scene(2).unwrap()));

    // Give our dispatcher some time to work
    while dispatcher.results_size() == 0 {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
 
    let mut result_vec = dispatcher.get_results().unwrap();

    let mut result_chunk = result_vec.pop().expect("error popping element off result");

    println!("Got result!: {:?}", result_chunk);

    // For now, don't verify and assume job was successful
    result_chunk.state = project::EncodeState::SPLIT;
    
    // Update our chunk
    testproject.update_chunk(result_chunk);

    // Push our first pass encode command
    dispatcher.push(commands::make_pass1(&testproject, testproject.get_scene(2).unwrap()));

    // Give our dispatcher some time to work
    while dispatcher.results_size() == 0 {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let mut result_vec = dispatcher.get_results().unwrap();

    let mut result_chunk = result_vec.pop().expect("error popping element off result");

    println!("Got result!: {:?}", result_chunk);

    result_chunk.state = project::EncodeState::PASS_1;

    // Update our chunk
    testproject.update_chunk(result_chunk);

    // Push our second pass encode command
    dispatcher.push(commands::make_pass2(&testproject, testproject.get_scene(2).unwrap()));

    // Give our dispatcher some time to work
    while dispatcher.results_size() == 0 {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let mut result_vec = dispatcher.get_results().unwrap();

    let mut result_chunk = result_vec.pop().expect("error popping element off result");

    println!("Got result!: {:?}", result_chunk);

    result_chunk.state = project::EncodeState::PASS_2;
    result_chunk.raw_file.clear();
    result_chunk.firstpass_file.clear();

    // Update our chunk
    testproject.update_chunk(result_chunk);

    dispatcher.finish();

    println!("saving project");
    testproject.save()

    
}
*/

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