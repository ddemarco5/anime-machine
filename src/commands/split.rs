use crate::commands;
use crate::project;

use std::io::{Write};
use std::process;
// for writing bat files
use std::fs::File;


// TODO!!!! Speed up the split! Possibly by using a duration arg in ffmpeg

const FFMPEG_SPLIT_BASE_COMMAND: &'static str = "{tools_path}\\ffmpeg.exe \
                                                -y \
                                                -i {source_file} \
                                                -vf select=\"between(n\\,{sf}\\,{ef}),setpts=PTS-STARTPTS\" \
                                                -pix_fmt yuv420p \
                                                {raws_path}\\{dest_file}";


#[derive(Debug, Clone)]  
pub struct Split {
    pub chunk: project::Chunk,
    pub cleanup_files: Vec<String>,
    video_source: String,
    // TODO: Don't copy the dirs, but store a refrence to the main project and read it from there
    dirs: project::Dirs,
}

impl Split {

    pub fn new(project: &project::Project, chunk: project::Chunk) -> Split {
        Split { 
            chunk: chunk.clone(),
            cleanup_files: Vec::new(),
            video_source: String::from(project.file_name.clone()),
            dirs: project.paths.clone(),
         }
    }

    fn build_batch_file(&mut self) -> String {

        // This is fuckin hacky. I need a better way to keep track of files
        self.chunk.raw_file = format!("split-{}-{}.y4m", self.chunk.start_frame, self.chunk.end_frame);

        let args = vec![
            ("tools_path", self.dirs.tools.clone()),
            ("source_file", self.video_source.clone()),
            ("sf", self.chunk.start_frame.to_string()),
            ("ef", self.chunk.end_frame.to_string()),
            ("raws_path", self.dirs.raw_chunks.clone()),
            ("dest_file", self.chunk.raw_file.clone())
        ];

        let batch_command = commands::build_batch_contents(FFMPEG_SPLIT_BASE_COMMAND, args);

        // make our filename and fill in our temp directory name
        let batch_filename = format!("{{tmp_dir}}\\split-{}-{}.bat", self.chunk.start_frame, self.chunk.end_frame)
                                    .replace("{tmp_dir}", &self.dirs.tmp.to_string());

        println!("DEBUG BATCH_FILENAME: {}", batch_filename);
        println!("DEBUG BATCH STRING: {}", batch_command);
               
        // create our file and write our batch string to it
        let mut file = File::create(batch_filename.clone()).expect("Error creating file");
        file.write_all(batch_command.as_bytes()).expect("Error writing file");
        // file close is handled by rust's scope awareness. Once we return, scope will be left, and file will be closed.
    
        return batch_filename;
    }
}

impl commands::Operation for Split {
    // Sets up all our requirements and returns our command handler
    fn prepare(&mut self) -> process::Command {

        println!("Working on sf: {}, ef: {}", self.chunk.start_frame, self.chunk.end_frame);

        // Create our batch file and record its location
        let batch_file_name = self.build_batch_file();
        println!("Filename: {}", batch_file_name.clone());
        self.cleanup_files.push(batch_file_name.clone());

        // Create our command
        println!("Creating command");
        let command = commands::build_start_command(batch_file_name);
        return command;
    }

    fn cleanup(&self) {
        println!("cleaning up after split!");
        for file in &self.cleanup_files {
            std::fs::remove_file(file.to_string()).expect("problem removing file in cleanup!");
        }
    }

    // Return a chunk that needs to be verified by the main project logic, consumes self
    fn getresults(&self) -> project::Chunk {
        return self.chunk.clone();
    }
}