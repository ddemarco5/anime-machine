// Top level caller of the rest of the code

use crate::project;
use crate::project::EncodeState::{NOT_STARTED, SPLIT, PASS_1, PASS_2, COMPLETE};
use crate::dispatcher;
use crate::commands;
use crate::commands::Operation;


pub struct Machine {
    project: project::Project,
    dispatcher: dispatcher::Dispatcher,
    work_list: Vec<project::Chunk>,
}

impl Machine {

    pub fn new(filename: String) -> Machine {
        Machine {
            project: project::Project::open(filename),
            dispatcher: dispatcher::Dispatcher::new(),
            work_list: Vec::new(),
        }
    }

    // Start the processing
    pub fn start(mut self, concurrent_chunks: usize) {

        // Inspect the chunk list of the project, report progress
        let num_chunks = self.project.chunks.len();
        let (done, notdone) = self.chunks_complete_incomplete();

        println!("{} total chunks, {} chunks done, {} left to go", num_chunks, done, notdone);

        // In the future: validate the state of each chunk by verifying files exist

        // Tell our dispatcher how many jobs we want to run at a time
        self.dispatcher.set_max_jobs(concurrent_chunks);

        // create a list of the first X number of chunks we can do work on
        //let mut work_list: Vec<project::Chunk> = Vec::new();

        match self.find_next_x_workable_chunks(concurrent_chunks) {
            // Add our vec of chunks to our work list
            Some(s) => {
                self.work_list.extend(s);
            },
            None => {
                println!("We couldn't find any other chunks to work on. Either project is done, or this is a bug");
            },
        }
        println!("Work list:\n{:?}", self.work_list);
        // for each chunk, submit the next command for its given state to the dispatcher
        for work_chunk in self.work_list.iter() {
            self.make_next_command(work_chunk).expect("We should not have unworkable chunks at this point");
        }

        // start our dispatcher
        self.dispatcher.start();

        // Cannot run finish immediately after start, because jobs won't get a chance to execute... rust is that fast
        //std::thread::sleep(std::time::Duration::from_secs(1));
        //self.dispatcher.finish();

        //std::process::exit(0);
        

        // in one big loop
        while !self.work_list.is_empty() {

            // Terminate if our chunk working list is empty

            // wait for the dispatcher to return a result
            println!("waiting for results");
            let results = self.dispatcher.wait_results();
            println!("got some results");

            // For each result
            for result in results {
                // Update the associated chunk in the project
                println!("Result chunk {:?} updated", result);
                self.project.update_chunk(result.clone());
                // save the project file so we don't lose work
                println!("Saving our project so we dont lose progress");
                self.project.save();
                // Find the next approprate job for the given chunk based on the new state
                match self.make_next_command(&result) {
                    // If such a job does exist, Create that job and submit it to the dispatcher
                    Some(_) => {
                        println!("Another job found, add it to the queue");
                    }
                    // If no such job exists, remove chunk from work list and pull the next
                    None => {
                        println!("This chunk is done");
                        // This is rust-ese for remove the result chunk from the work list
                        self.work_list.retain(|r| r.scene_number != result.scene_number);
                        // get the next workable chunk and add it
                        match self.find_next_x_workable_chunks(1) { 
                            Some(c) => {
                                println!("Found another workable chunk: {:?}", c[0]);
                                self.work_list.push(c[0].clone()); // we know we're only getting 1
                                // and work on it
                                self.make_next_command(&c[0]).expect("No command for chunk, there is a bug somewhere");
                            }
                            // If no more chunks exist, don't add one
                            None => {
                                println!("No more workable chunks in project");
                            }
                        }

                    }
                }
                
            }

            // Inspect the chunk list of the project, report progress, really just for debug/notification purposes
            let num_chunks = self.project.chunks.len();
            let (done, notdone) = self.chunks_complete_incomplete();
            println!("{} total chunks, {} chunks done, {} left to go", num_chunks, done, notdone);

        }

        println!("Nothing left in our work list, we must not have any more chunks to work on!");

        // Merge the file here
        let mut mergejob = commands::make_merge(&self.project);

        println!("Merging the encoded chunks");
        let mut mergecommand = mergejob.prepare();

        mergecommand.output().expect("failed to execute merge job");

        // don't clean right now for debugging purposes
        mergejob.cleanup();

        // Mark all of our chunks as "complete"
        println!("Marking chunks as complete");
        for chunk in self.project.chunks.iter_mut() {
            chunk.state = COMPLETE;
        }

        self.project.save();

        self.dispatcher.finish();

    }



    // Create the next approprate command based on our current chunk state
    fn make_next_command(&self, chunk: &project::Chunk) -> Option<()> {

        match &chunk.state {
            NOT_STARTED => {
                println!("Submitting split command!");
                self.dispatcher.push(commands::make_split(&self.project, chunk.clone()));
                return Some(());
            }
            SPLIT => {
                println!("Submitting pass1 command!");
                self.dispatcher.push(commands::make_pass1(&self.project, chunk.clone()));
                return Some(());
            }
            PASS_1 => {
                println!("Submitting pass2 command!");
                self.dispatcher.push(commands::make_pass2(&self.project, chunk.clone()));
                return Some(());
            }
            // Anything else is marked as completed or pass2, effectively the same thing
            _ => return None,
        }

    }

    fn find_next_x_workable_chunks(&self, chunks_needed: usize) -> Option<Vec<project::Chunk>> {
        let mut found_chunks: Vec<project::Chunk> = Vec::new();

        for chunk in self.project.chunks.iter() {
            if (chunk.state != COMPLETE) && (chunk.state != PASS_2) && !self.find_chunk_in_work_list(&chunk) {
                found_chunks.push(chunk.clone());
            }
            if found_chunks.len() == chunks_needed {
                break;
            }
        }

        if found_chunks.is_empty() {
            return None;
        }

        println!("Found {} chunks to work", found_chunks.len());
        return Some(found_chunks);

    }

    fn find_chunk_in_work_list(&self, match_chunk: &project::Chunk) -> bool {

        // Loop through our work list looking for the chunk in question
        for chunk in self.work_list.iter() {
            if chunk.scene_number == match_chunk.scene_number {
                return true;
            }
        }
        // If not found, return false
        return false;

    }

    fn chunks_complete_incomplete(&self) -> (usize, usize) {
        let mut done = 0;
        let mut notdone = 0;
        for chunk in self.project.chunks.iter() {
            match chunk.state {
                COMPLETE => done += 1,
                PASS_2 => done += 1,
                _ => notdone += 1,
            }
        }
        return (done, notdone);
    }


}