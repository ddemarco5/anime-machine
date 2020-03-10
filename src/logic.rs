// Top level caller of the rest of the code

use crate::project;
use crate::project::EncodeState;
use crate::dispatcher;
use crate::commands;


pub struct Machine {
    project: project::Project,
    dispatcher: dispatcher::Dispatcher,
}

impl Machine {

    pub fn new(filename: String) -> Machine {
        Machine {
            project: project::Project::open(filename),
            dispatcher: dispatcher::Dispatcher::new(),
        }
    }

    // Start the processing
    pub fn start(mut self, concurrent_chunks: usize) {

        // Inspect the chunk list of the project, report progress
        let num_chunks = self.project.chunks.len();
        let (done, notdone) = self.project.chunks_complete_incomplete();

        println!("{} total chunks, {} chunks done, {} left to go", num_chunks, done, notdone);

        // In the future: validate the state of each chunk by verifying files exist

        // start our dispatcher
        self.dispatcher.start();
        self.dispatcher.set_max_jobs(concurrent_chunks);

        // create a list of the first X number of chunks we can do work on
        let mut work_list: Vec<project::Chunk> = Vec::new();
        match self.find_next_X_workable_chunks(concurrent_chunks) {
            // Add our vec of chunks to our work list
            Some(s) => {
                work_list.extend(s);
            },
            None => {
                panic!("We couldn't find any other chunks to work on. Might be a bug if this hits")
            },
        }
        println!("Work list:\n{:?}", work_list);
        // for each chunk, submit the next command for its given state to the dispatcher
        for work_chunk in work_list {
            self.make_next_command(work_chunk).expect("We should not have unworkable chunks at this point");
            //self.dispatcher.push(*command);
        }

        // TESTING: Wait a few seconds for our job to get picked up
        std::thread::sleep(std::time::Duration::from_secs(8));
        self.dispatcher.finish();
        

        // in one big loop

            // Terminate if our chunk working list is empty

            // wait for the dispatcher to return a result
                // For each result
                    // Update the associated chunk in the project
                    // save the project file so we don't lose work
                    // Find the next approprate job for the given chunk based on the new state
                        // If no such job exists, remove chunk from work list and pull the next
                            // If no more chunks exist, don't add one
                        // If such a job does exist, Create that job and submit it to the dispatcher

    }

    // Create the next approprate command based on our current chunk state
    fn make_next_command(&self, chunk: project::Chunk) -> Option<()> {

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

    fn find_next_X_workable_chunks(&self, chunks_needed: usize) -> Option<Vec<project::Chunk>> {
        let mut found_chunks: Vec<project::Chunk> = Vec::new();

        for chunk in self.project.chunks.iter() {
            if chunk.state != project::EncodeState::COMPLETE {
                found_chunks.push(chunk.clone());
            }
            if found_chunks.len() == chunks_needed {
                break;
            }
        }

        if found_chunks.is_empty() {
            return None;
        }

        return Some(found_chunks);

    }


}