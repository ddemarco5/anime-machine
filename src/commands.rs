use std::process;

mod split;
mod encode;
mod mediainfo;
mod merge;

use crate::project;


const BATCH_BASE_STRING: &'static str = "@echo off\n\
                                        {command}\n\
                                        exit";


pub trait Operation {
    fn prepare(&mut self) -> process::Command;
    fn cleanup(&self);
    fn getresults(&self) -> project::Chunk;
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub chunk_num: usize,
    pub file_name: String,
    pub start_frame: usize,
    pub end_frame: usize,
}

pub fn make_split(project: &project::Project, chunk: project::Chunk) -> split::Split {
    return split::Split::new(project, chunk);
}

pub fn make_pass1(project: &project::Project, chunk: project::Chunk) -> encode::EncodePass1 {
    return encode::EncodePass1::new(project, chunk);
}

pub fn make_pass2(project: &project::Project, chunk: project::Chunk) -> encode::EncodePass2 {
    return encode::EncodePass2::new(project, chunk);
}

pub fn make_mediainfo(project: &project::Project, target: String) -> mediainfo::MediaInfo {
    return mediainfo::MediaInfo::new(project, target);
}

pub fn make_merge(project: &project::Project) -> merge::Merge {
    return merge::Merge::new(project);
}


pub fn build_start_command(batchfile: String) -> process::Command {

    //let filename = build_ffmpeg_batch_file(filename, start_frame, end_frame);

    let mut command = process::Command::new("cmd");
    command.arg("/C");
    command.args(&["start", "/WAIT", &batchfile.to_string()]);


    return command;
}

pub fn build_batch_contents(command_template: &str, arguments: Vec<(&str, String)>) -> String {

    let mut filled_command = command_template.clone().to_string();

    for (name, value) in arguments {
        // weird, but {{ escapes a curly brace, and we want '{name}' as a result
        let matchstr = format!("{{{}}}", name);
        filled_command = filled_command.replace(&matchstr.to_string(), &value.to_string());
    }

    let final_batch_command = BATCH_BASE_STRING.replace("{command}", &filled_command.to_string());

    return String::from(final_batch_command);

}
