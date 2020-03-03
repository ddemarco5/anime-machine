use std::process;
use std::io::{self, Write};

// for our loop waits
use std::thread::sleep;
use std::time::Duration;

// for writing bat files because the command line is ridiculous with special characters
use std::fs::File;

const FFMPEG_SPLIT_BASE_COMMAND: &'static str = "@echo off\n\
                                                ffmpeg.exe \
                                                -y \
                                                -i {target_file} \
                                                -vf select=\"between(n\\,{sf}\\,{ef}),setpts=PTS-STARTPTS\" \
                                                -pix_fmt yuv420p \
                                                split-{sf}-{ef}.y4m\n\
                                                exit";


pub trait Operation {
    fn execute(&self);
    fn prepare(&mut self) -> process::Command;
    fn cleanup(&self);
    fn getresults(&self) -> (JobInfo, Vec<String>);
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub chunk_num: usize,
    pub file_name: String,
    pub start_frame: usize,
    pub end_frame: usize,
}

#[derive(Debug, Clone)]  
pub struct Split {
    info: JobInfo,
    cleanup_files: Vec<String>,
    raw_output: String,
}

// TODO: finish me
#[derive(Debug, Clone)]  
pub struct Encode {
    info: JobInfo,
    crap: usize,
}

impl Operation for Split {
    fn execute(&self) {
        println!("Split command executing!");
        ffmpeg_split_chunks(&self.info.file_name.to_string(), vec![(self.info.start_frame, self.info.end_frame)]);
    }

    // Sets up all our requirements and returns our command handler
    fn prepare(&mut self) -> process::Command {
        let sf = self.info.start_frame;
        let ef = self.info.end_frame;
        let filestr = &self.info.file_name.to_string();

        println!("Working on sf: {}, ef: {}", sf, ef);

        // Create and record our batch file
        let batch_file_name = build_ffmpeg_batch_file(filestr, sf, ef);
        println!("Filename: {}", batch_file_name.clone());
        self.cleanup_files.push(batch_file_name.clone());

        self.raw_output = batch_file_name.replace(".bat", ".y4m");

        // Create and record our command
        println!("Creating command");
        let command = build_ffmpeg_split_command(batch_file_name);
        return command;
    }

    fn cleanup(&self) {
        println!("cleaning up after split!");
        for file in &self.cleanup_files {
            std::fs::remove_file(file.to_string()).expect("problem removing file in cleanup!");
        }
    }

    fn getresults(&self) -> (JobInfo, Vec<String>) {
        return (self.info.clone(), vec![self.raw_output.clone()]);
    }
}

impl Operation for Encode {
    fn execute(&self) {
        println!("encode command would execute here!");
    }
    fn prepare(&mut self) -> process::Command {
        return process::Command::new("dir");
    }
    fn cleanup(&self) {
        println!("write me, tehee");
    }
    fn getresults(&self) -> (JobInfo, Vec<String>) {
        return (self.info.clone(), vec![String::new()]);
    }
}

pub fn test_split(chunknum: usize, startframe: usize, endframe: usize) -> Split {
    return Split { 
        info: JobInfo {
            chunk_num: chunknum,
            file_name: String::from("ep1-vid.mkv"),
            start_frame: startframe,
            end_frame: endframe,
        },
        cleanup_files: Vec::new(),
        raw_output: String::new(),
     };
}

pub fn test_encode() -> Encode {
    return Encode { 
        info: JobInfo {
            chunk_num: 0,
            file_name: String::from("ep1-vid.mkv"),
            start_frame: 0,
            end_frame: 100,
        },
        crap: 0,
     };
}

fn build_base_command() -> process::Command {
    let mut command = process::Command::new("cmd");
    command.arg("/C");
    return command;
}

// This is the entire logic loop for executing a batch of ffmpeg split commands.
// It might be smart to move them to another class/file to avoid bloat in this one
pub fn ffmpeg_split_chunks(filename: &str, frame_pairs: Vec<(usize,usize)>) -> Vec<String> {

    let mut bash_filenames: Vec<String> = Vec::new();
    let mut raw_filenames: Vec<String> = Vec::new();
    let mut commands: Vec<process::Command> = Vec::new();

    // Create all our batch files and commands we'll be executing
    for (sf, ef) in frame_pairs {
        println!("Working on sf: {}, ef: {}", sf, ef);

        // Create and record our batch file
        let name = build_ffmpeg_batch_file(filename, sf, ef);
        println!("Filename: {}", name.clone());
        bash_filenames.push(name.clone());

        raw_filenames.push(name.replace(".bat", ".y4m"));

        // Create and record our command
        println!("Creating command");
        let command = build_ffmpeg_split_command(name.clone());
        commands.push(command);
    }

    println!("we now have a file vec of size: {}", bash_filenames.len());
    println!("and a command vec of size: {}", commands.len());
    
    //Execute our commands
    async_batch_execute(&mut commands);

    //Delete the batch files we are finished executing
    for filename in bash_filenames {
        std::fs::remove_file(&filename.to_string()).expect("Failed to delete file");
        println!("Deleted file: {}", filename);
    }
    return raw_filenames;
}


fn build_ffmpeg_batch_file(filename: &str, start_frame: usize, end_frame: usize) -> String {

    let batch_filename = format!("split-{}-{}.bat", start_frame, end_frame);

    let command = FFMPEG_SPLIT_BASE_COMMAND.replace("{target_file}", filename)
                                           .replace("{sf}", &start_frame.to_string())
                                           .replace("{ef}", &end_frame.to_string());

                                         
    let mut file = File::create(batch_filename.clone()).expect("Error creating file");
    file.write_all(command.as_bytes()).expect("Error writing file");
    // file close is handled by rust's scope awareness. Once we return, scope will be left, and file will be closed.

    return batch_filename;
}
    

fn build_ffmpeg_split_command(batchfile: String) -> process::Command {

    //let filename = build_ffmpeg_batch_file(filename, start_frame, end_frame);

    let mut command = build_base_command();
    command.args(&["start", "/WAIT", &batchfile.to_string()]);

    return command;
}

fn async_batch_execute(commands_to_run: &mut Vec<process::Command>) {

    let mut childlist: Vec<process::Child> = Vec::new();

    for command in commands_to_run {
        let child = command.spawn().expect("failed to async batch execute");
        let pid = child.id();
        childlist.push(child);
        println!("Spawned pid: {}", pid);
    }

    process_waiter(childlist);

}

// blocking loop that will wait for process termination
fn process_waiter(mut child_list: Vec<process::Child>) {

    let mut counter = 0;

    while !child_list.is_empty() {

        let mut kid_removed = false;

        // print a little update dot every 5 seconds
        if counter % 5 == 0 {
            print!(".");
            io::stdout().flush().unwrap();
        }

        // loop to find kiddos to delete
        for (i, child) in child_list.iter_mut().enumerate() {

            // If our child is terminated
            // remove_item isn't in yet, so we iterate with enumerate
            if child.try_wait().unwrap() != None { 
                println!("Removing child pid: {}, index {}", child.id(), i);
                // remove it from the list
                child_list.remove(i);
                kid_removed = true;
                break; // git outta here, we can't delete any more if we just removed one
            }
        }

        // Skip the wait if a child was removed, there might another one, and we have to start from scratch since the vec
        // remove option also resizes, and we'll hit a bounds panic if we try to remove multiples
        if !kid_removed{
            //wait for 1 second
            sleep(Duration::from_secs(1));
            counter += 1;
        }
    }
}