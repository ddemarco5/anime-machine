use crate::commands;
use crate::project;

use std::process;

// for creating the batch file
use std::fs::File;
use std::io::{Write};

// -o should be the ivf file, last arg should be the raw y4m file
const AOMENC_BASE_COMMAND: &'static str = "{tools_path}\\aomenc.exe \
                                           -v --ivf --passes=2 \
                                           --pass={pass_num} \
                                           --fpf=\"{enc_path}\\{passfile_name}\" \
                                           --cpu-used=3 --end-usage=q --cq-level=30 \
                                           --tile-rows=0 --tile-columns=0 --threads=2 \
                                           --tune=psnr --rate-hist=5 --q-hist=5 \
                                           -o {enc_path}\\{encoded_filename} \
                                           {raw_path}\\{raw_chunk_filename}";



// This file will hold both of our encoding pass commands, as they're basically identical
#[derive(Debug, Clone)]  
pub struct EncodePass1 {
    chunk: project::Chunk,
    cleanup_files: Vec<String>,
    dirs: project::Dirs,
}

#[derive(Debug, Clone)]  
pub struct EncodePass2 {
    chunk: project::Chunk,
    cleanup_files: Vec<String>,
    dirs: project::Dirs,
}

impl EncodePass1 {
    pub fn new(project: &project::Project, chunk: project::Chunk) -> EncodePass1 {
        EncodePass1 {
            chunk: chunk.clone(),
            cleanup_files: Vec::new(),
            dirs: project.paths.clone(),
        }
    }

    fn build_batch_file (&mut self) -> String {

        self.chunk.encoded_file = format!("{}-firstpass", self.chunk.raw_file.clone());

        let args = vec![
            ("tools_path", self.dirs.tools.clone()),
            ("pass_num", "1".to_string()),
            ("passfile_name", self.chunk.encoded_file.clone()),
            ("enc_path", self.dirs.encoded_chunks.clone()),
            ("encoded_filename", self.chunk.raw_file.replace(".y4m", ".ivf")),
            ("raw_path", self.dirs.raw_chunks.clone()),
            ("raw_chunk_filename", self.chunk.raw_file.clone())
        ];

        let batch_command = commands::build_batch_contents(AOMENC_BASE_COMMAND,args);

        let batch_filename = format!("{{tmp_dir}}\\enc1-{}.bat", self.chunk.raw_file)
                                    .replace("{tmp_dir}", &self.dirs.tmp.to_string());

        println!("DEBUG BATCH_FILENAME: {}", batch_filename);
        println!("DEBUG BATCH STRING: {}", batch_command);

         // create our file and write our batch string to it
         let mut file = File::create(batch_filename.clone()).expect("Error creating file");
         file.write_all(batch_command.as_bytes()).expect("Error writing file");

         return batch_filename;
    }

}

impl EncodePass2 {
    pub fn new(project: &project::Project, chunk: project::Chunk) -> EncodePass2 {
        EncodePass2 {
            chunk: chunk.clone(),
            cleanup_files: Vec::new(),
            dirs: project.paths.clone(),
        }
    }

    fn build_batch_file (&mut self) -> String {

        let firstpass_file = self.chunk.encoded_file.clone();
        self.chunk.encoded_file = self.chunk.raw_file.replace(".y4m", ".ivf");

        let args = vec![
            ("tools_path", self.dirs.tools.clone()),
            ("pass_num", "2".to_string()),
            ("passfile_name", firstpass_file),
            ("enc_path", self.dirs.encoded_chunks.clone()),
            ("encoded_filename", self.chunk.encoded_file.clone()),
            ("raw_path", self.dirs.raw_chunks.clone()),
            ("raw_chunk_filename", self.chunk.raw_file.clone())
        ];

        let batch_command = commands::build_batch_contents(AOMENC_BASE_COMMAND,args);

        let batch_filename = format!("{{tmp_dir}}\\enc1-{}.bat", self.chunk.raw_file)
                                    .replace("{tmp_dir}", &self.dirs.tmp.to_string());

        println!("DEBUG BATCH_FILENAME: {}", batch_filename);
        println!("DEBUG BATCH STRING: {}", batch_command);

         // create our file and write our batch string to it
         let mut file = File::create(batch_filename.clone()).expect("Error creating file");
         file.write_all(batch_command.as_bytes()).expect("Error writing file");

         return batch_filename;
    }

}

impl commands::Operation for EncodePass1 {

    fn prepare(&mut self) -> process::Command {
        // Create our batch file and save its location
        let batch_file_name = self.build_batch_file();
        self.cleanup_files.push(batch_file_name.clone());

        // Create our command
        let command = commands::build_start_command(batch_file_name);
        return command;
    }
    fn cleanup(&self) {
        println!("cleaning up after enc1!");
        for file in &self.cleanup_files {
            std::fs::remove_file(file.to_string()).expect("problem removing file in cleanup!");
        }
    }
    fn getresults(&self) -> project::Chunk {
        return self.chunk.clone();
    }
}

impl commands::Operation for EncodePass2 {

    fn prepare(&mut self) -> process::Command {

        // On the second pass we'll no longer need the raw or the firstpass data
        self.cleanup_files.push(self.dirs.raw_chunks.clone() + "\\" + &self.chunk.raw_file.clone());
        self.cleanup_files.push(self.dirs.encoded_chunks.clone() + "\\" + &self.chunk.encoded_file.clone());

        // Create our batch file and save its location
        let batch_file_name = self.build_batch_file();
        self.cleanup_files.push(batch_file_name.clone());

        // Create our command
        let command = commands::build_start_command(batch_file_name);
        return command;
    }
    fn cleanup(&self) {
        println!("cleaning up after enc2!");
        for file in &self.cleanup_files {
            std::fs::remove_file(file.to_string()).expect("problem removing file in cleanup!");
        }
    }
    fn getresults(&self) -> project::Chunk {
        return self.chunk.clone();
    }
}