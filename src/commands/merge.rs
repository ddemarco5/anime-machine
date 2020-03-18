use crate::commands;
use crate::project;

use std::io::{Write};
use std::process;
// for writing bat files
use std::fs::File;

const MKVMERGE_BASE_COMMAND: &'static str = "{tools_path}\\mkvtoolnix\\mkvmerge.exe \
                                             -o {output_file} \
                                             --timestamps 0:{timestamps_file} \
                                             {encoded_file_list} \
                                             -D {original_file}";

#[derive(Debug, Clone)]  
pub struct Merge<'a> {
    timestamp_file: String,
    pub cleanup_files: Vec<String>,
    project: &'a project::Project,
}

impl Merge<'_> {

    pub fn new(project: &project::Project) -> Merge {
        Merge { 
            timestamp_file: String::new(),
            cleanup_files: Vec::new(),
            project: project,
         }
    }

    fn build_file_list(&self) -> String {
        let pathskelly = String::from("{enc_path}/{filename}").replace("{enc_path}", &self.project.paths.encoded_chunks.to_string());
        let mut returnstring = String::new();
        for chunk in self.project.chunks.iter() {
            if !returnstring.is_empty() {
                returnstring.push_str(" +");
            }
            returnstring.push_str(&pathskelly.clone().replace("{filename}", &chunk.encoded_file.to_string()));
        }
        return returnstring;
    }

    fn build_timestamp_file(&mut self) {
        let path = "{tmp_path}\\timestamps.txt".replace("{tmp_path}", &self.project.paths.tmp.clone());
        let mut file = File::create(path.clone()).expect("Error opening timestamps file for writing");

        // Write the necessary headers
        file.write_all(b"# timestamp format v1\n").expect("Error writing timestamp file");
        let line = "assume {fps}\n".replace("{fps}", &self.project.get_fps().to_string());
        file.write_all(line.as_bytes()).expect("Error writing timestamp file");

        let skelly = "{sf},{ef},{fps}\n".replace("{fps}", &self.project.get_fps().to_string());
        // Loop and add the needed lines to the timestamp file
        for chunk in self.project.chunks.iter() {
            let line = skelly.clone().replace("{sf}", &chunk.start_frame.to_string())
                                    .replace("{ef}", &chunk.end_frame.to_string());
            file.write_all(line.as_bytes()).expect("Error writing timestamp file");
        }

        self.timestamp_file = path;
    }

    fn build_batch_file(&mut self) -> String {

        // build list of files to append
        let file_list = self.build_file_list();
        // build our timestamp file
        self.build_timestamp_file();

        let mut outfile = self.project.file_name.clone();
        outfile.push_str(".encoded.mkv"); // temporary for testing
        let args = vec![
            ("tools_path", self.project.paths.tools.clone()),
            ("output_file", outfile), // temporary
            ("timestamps_file", self.timestamp_file.clone()),
            ("encoded_file_list", file_list),
            ("original_file", self.project.file_name.clone()),
        ];

        let batch_command = commands::build_batch_contents(MKVMERGE_BASE_COMMAND, args);

        // make our filename and fill in our temp directory name
        let batch_filename = "{tmp_dir}\\mkvmerge.bat".replace("{tmp_dir}", &self.project.paths.tmp.to_string());

        println!("DEBUG BATCH_FILENAME: {}", batch_filename);
        println!("DEBUG BATCH STRING: {}", batch_command);
               
        // create our file and write our batch string to it
        let mut file = File::create(batch_filename.clone()).expect("Error creating file");
        file.write_all(batch_command.as_bytes()).expect("Error writing file");
        // file close is handled by rust's scope awareness. Once we return, scope will be left, and file will be closed.
    
        return batch_filename;
    }
}

impl commands::Operation for Merge<'_> {
    // Sets up all our requirements and returns our command handler
    fn prepare(&mut self) -> process::Command {

        // Create our batch file and record its location
        let batch_file_name = self.build_batch_file();
        println!("Filename: {}", batch_file_name.clone());
        self.cleanup_files.push(batch_file_name.clone());
        self.cleanup_files.push(self.timestamp_file.clone());

        // Create our command
        println!("Creating command");
        let command = commands::build_start_command(batch_file_name);
        return command;
    }

    fn cleanup(&self) {
        println!("cleaning up after mkvmerge!");
        for file in &self.cleanup_files {
            std::fs::remove_file(file.to_string()).expect("problem removing file in cleanup!");
        }
    }

    // stub for the mediainfo command
    fn getresults(&self) -> project::Chunk {
        panic!("Don't call this");
    }
}