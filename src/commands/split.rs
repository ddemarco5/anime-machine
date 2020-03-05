use crate::commands;
use crate::project;

use std::io::{Write};
use std::process;
// for writing bat files
use std::fs::File;


const FFMPEG_SPLIT_BASE_COMMAND: &'static str = "{tools_path}\\ffmpeg.exe \
                                                -y \
                                                -i {target_file} \
                                                -vf select=\"between(n\\,{sf}\\,{ef}),setpts=PTS-STARTPTS\" \
                                                -pix_fmt yuv420p \
                                                {raws_path}\\split-{sf}-{ef}.y4m";


#[derive(Debug, Clone)]  
pub struct Split {
    pub info: commands::JobInfo,
    pub cleanup_files: Vec<String>,
    pub raw_output: String,
    video_source: String,
    dirs: project::Dirs,
}

impl commands::Operation for Split {
    // Sets up all our requirements and returns our command handler
    fn prepare(&mut self) -> process::Command {
        let sf = self.info.start_frame;
        let ef = self.info.end_frame;

        println!("Working on sf: {}, ef: {}", sf, ef);

        // Create and record our batch file
        let batch_file_name = self.build_batch_file();
        println!("Filename: {}", batch_file_name.clone());
        self.cleanup_files.push(batch_file_name.clone());

        //self.raw_output = batch_file_name.replace(".bat", ".y4m");

        // Create and record our command
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

    fn getresults(&self) -> (commands::JobInfo, Vec<String>) {
        return (self.info.clone(), vec![self.raw_output.clone()]);
    }
}

impl Split {

    pub fn new(project: &project::Project, chunk: &project::Chunk) -> Split {
        Split { 
            info: commands::JobInfo {
                chunk_num: chunk.scene_number,
                file_name: project.file_name.clone(),
                start_frame: chunk.start_frame,
                end_frame: chunk.end_frame,
            },
            cleanup_files: Vec::new(),
            raw_output: String::new(),
            video_source: String::from(project.file_name.clone()),
            dirs: project.paths.clone(),
         }
    }

    // TODO: In the future this should probably be generalize and pushed up one scope into the commands.rs file
    // As in todo tomorrow... If I want to add more commands this has got to be reusable code
    fn build_batch_file(&mut self) -> String {

        let mut batch_filename = format!("{{tmp_dir}}\\split-{}-{}.bat", self.info.start_frame, self.info.end_frame);

        //TODO: This is fuckin hacky. We need a better way to keep track of tiles
        self.raw_output = format!("split-{}-{}.y4m", self.info.start_frame, self.info.end_frame);


        println!("DEBUG: {}", batch_filename);
        println!("DEBUG TOOLS: {}", &self.dirs.tools.to_string());

        batch_filename = batch_filename.replace("{tmp_dir}", &self.dirs.tmp.to_string());

        //let batch_filename = format!("split-{}-{}.bat", self.info.start_frame, self.info.end_frame);
    
        let command = FFMPEG_SPLIT_BASE_COMMAND.replace("{tools_path}", &self.dirs.tools.to_string())
                                               .replace("{target_file}", &self.info.file_name.to_string())
                                               .replace("{sf}", &self.info.start_frame.to_string())
                                               .replace("{ef}", &self.info.end_frame.to_string())
                                               .replace("{raws_path}", &self.dirs.raw_chunks.to_string());
    
        let batch_string = commands::BATCH_BASE_STRING.replace("{command}", &command.to_string());


        println!("DEBUG: {}", batch_filename);
        println!("DEBUG BATCH STRING: {}", batch_string);
                                             
        let mut file = File::create(batch_filename.clone()).expect("Error creating file");
        file.write_all(batch_string.as_bytes()).expect("Error writing file");
        // file close is handled by rust's scope awareness. Once we return, scope will be left, and file will be closed.
    
        return batch_filename;
    }
}