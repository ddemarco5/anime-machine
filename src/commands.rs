use std::process;

mod split;
mod encode;

use crate::project;


pub trait Operation {
    //fn execute(&self);
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

/*
pub fn test_split(chunknum: usize, startframe: usize, endframe: usize) -> split::Split {
    return split::Split { 
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
*/

pub fn make_split(project: &project::Project, chunk: &project::Chunk) -> split::Split {
    return split::Split::new(project, chunk);
}

pub fn test_encode() -> encode::Encode {
    return encode::Encode { 
        info: JobInfo {
            chunk_num: 0,
            file_name: String::from("ep1-vid.mkv"),
            start_frame: 0,
            end_frame: 100,
        },
        crap: 0,
     };
}

pub fn build_start_command(batchfile: String) -> process::Command {

    //let filename = build_ffmpeg_batch_file(filename, start_frame, end_frame);

    let mut command = process::Command::new("cmd");
    command.arg("/C");
    command.args(&["start", "/WAIT", &batchfile.to_string()]);


    return command;
}
