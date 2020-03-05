use crate::commands;
use crate::project;

use std::process;

// -o should be the ivf file, last arg should be the raw y4m file
const AOMENC_BASE_COMMAND: &'static str = "{tools_path}\\aomenc.exe \
                                           -v --ivf --passes=2 \
                                           --pass={pass_num} \
                                           --fpf=\"{tmp_path}\\{chunk_name}-firstpass\" \
                                           --cpu-used=3 --end-usage=q --cq-level=30 \
                                           --tile-rows=0 --tile-columns=0 --threads=2 \
                                           --tune=psnr --rate-hist=5 --q-hist=5 \
                                           -o {encoded_filename} \
                                           {raw_chunk_filename}";



// This file will hold both of our encoding pass commands, as they're basically identical
#[derive(Debug, Clone)]  
pub struct EncodePass1 {
    pub info: commands::JobInfo,
    pub crap: usize,
    pub cleanup_files: Vec<String>,
    video_source: String,
    dirs: project::Dirs,
}

#[derive(Debug, Clone)]  
pub struct EncodePass2 {
    pub info: commands::JobInfo,
    pub crap: usize,
    video_source: String,
    dirs: project::Dirs,
}

impl commands::Operation for EncodePass1 {

    fn prepare(&mut self) -> process::Command {
        return process::Command::new("dir");
    }
    fn cleanup(&self) {
        println!("write me, tehee");
    }
    fn getresults(&self) -> (commands::JobInfo, Vec<String>) {
        return (self.info.clone(), vec![String::new()]);
    }
}

impl commands::Operation for EncodePass2 {
    

    fn prepare(&mut self) -> process::Command {
        return process::Command::new("dir");
    }
    fn cleanup(&self) {
        println!("write me, tehee");
    }
    fn getresults(&self) -> (commands::JobInfo, Vec<String>) {
        return (self.info.clone(), vec![String::new()]);
    }
}