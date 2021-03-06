use crate::commands;
use crate::project;

use std::io::{Write};
use std::process;
// for writing bat files
use std::fs::File;

const MEDIAINFO_BASE_COMMAND: &'static str = "{tools_path}\\mediainfo\\MediaInfo.exe \
                                             -f \
                                             --Output=JSON \
                                             --LogFile={tmp_path}\\{target_file}.json \
                                             {target_file}";

#[derive(Debug, Clone)]  
pub struct MediaInfo {
    target_file: String,
    pub cleanup_files: Vec<String>,
    video_source: String,
    // TODO: Don't copy the dirs, but store a refrence to the main project and read it from there
    dirs: project::Dirs,
}

impl MediaInfo {

    pub fn new(project: &project::Project, target: String) -> MediaInfo {
        MediaInfo { 
            target_file: target,
            cleanup_files: Vec::new(),
            video_source: String::from(project.file_name.clone()),
            dirs: project.paths.clone(),
         }
    }

    fn build_batch_file(&mut self) -> String {

        let args = vec![
            ("tools_path", self.dirs.tools.clone()),
            ("tmp_path", self.dirs.tmp.clone()),
            ("target_file", self.target_file.clone()),
        ];

        let batch_command = commands::build_batch_contents(MEDIAINFO_BASE_COMMAND, args);

        // make our filename and fill in our temp directory name
        let batch_filename = "{tmp_dir}\\mediainfo.bat".replace("{tmp_dir}", &self.dirs.tmp.to_string());

        println!("DEBUG BATCH_FILENAME: {}", batch_filename);
        println!("DEBUG BATCH STRING: {}", batch_command);
               
        // create our file and write our batch string to it
        let mut file = File::create(batch_filename.clone()).expect("Error creating file");
        file.write_all(batch_command.as_bytes()).expect("Error writing file");
        // file close is handled by rust's scope awareness. Once we return, scope will be left, and file will be closed.
    
        return batch_filename;
    }
}

impl commands::Operation for MediaInfo {
    // Sets up all our requirements and returns our command handler
    fn prepare(&mut self) -> process::Command {

        // Create our batch file and record its location
        let batch_file_name = self.build_batch_file();
        println!("Filename: {}", batch_file_name.clone());
        self.cleanup_files.push(batch_file_name.clone());
        self.cleanup_files.push("{tmp_dir}\\{target_file}.json"
                                .replace("{tmp_dir}", &self.dirs.tmp.to_string())
                                .replace("{target_file}", &self.target_file.to_string()));

        // Create our command
        println!("Creating command");
        let command = commands::build_start_command(batch_file_name);
        return command;
    }

    fn cleanup(&self) {
        println!("cleaning up after mediainfo!");
        for file in &self.cleanup_files {
            std::fs::remove_file(file.to_string()).expect("problem removing file in cleanup!");
        }
    }

    // stub for the mediainfo command
    fn getresults(&self) -> project::Chunk {
        panic!("Don't call this");
    }
}