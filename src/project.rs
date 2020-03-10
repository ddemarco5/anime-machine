use serde::Serialize;
use serde::Deserialize;
use std::fs;
use std::path;

use crate::scene_parse;
use crate::commands;
use crate::dispatcher;

const PROJECT_FILE: &'static str = "project.yaml";

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum EncodeState {
    NOT_STARTED,
    SPLIT,
    PASS_1,
    PASS_2,
    COMPLETE
}

#[derive(Deserialize, Serialize)]
pub struct Project{
    pub file_name: String,
    pub paths: Dirs,
    pub chunks: Vec<Chunk>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Chunk {
    pub scene_number: usize,
    pub start_frame: usize,
    pub end_frame: usize,
    pub length_frames: usize,
    pub state: EncodeState,
    pub raw_file: String,
    pub firstpass_file: String,
    pub encoded_file: String,
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Dirs {
    pub base: String,
    pub tools: String,
    pub tmp: String,
    pub raw_chunks: String,
    pub encoded_chunks: String,
}

impl Project {

    fn new() -> Project {
        Project { 
            file_name: String::new(),
            paths: Dirs {
                base: String::new(),
                tools: String::from("tools"),
                tmp: String::from("tmp"),
                raw_chunks: String::from("raws"),
                encoded_chunks: String::from("enc"),
            },
            chunks: Vec::new(),
        }
    }

    pub fn open(filename: String) -> Project {

        // Could handle the case of project file not existing, but if we think it exists and doesn't
        // that means there's a bug somewhere else, so we'll panic
        if !std::path::Path::new(PROJECT_FILE).exists() {
            println!("Project file doesn't exist! Creating a new one");
            let mut project = Project::new();
            project.file_name = filename;
            let parsed_scenes = scene_parse::build_records("ep1-vid-Scenes.csv").unwrap();
            println!("parsed {} lines from scene file.", parsed_scenes.len());
            for record in parsed_scenes {
                project.add_chunk(record);
            }
            return project;
        }
        else {
            let reader = fs::File::open(PROJECT_FILE).expect("Problem opening project file!");
            let project = match serde_yaml::from_reader(reader) {
                Err(error) => panic!("Error reading project! It might be old. If it is, delete the yaml file\n{:?}", error),
                Ok(project) => project,
            };
            println!("Successfully read project file!");
            return project;
        }
    }

    pub fn chunks_complete_incomplete(&self) -> (usize, usize) {
        let mut done = 0;
        let mut notdone = 0;
        for chunk in self.chunks.iter() {
            match chunk.state {
                EncodeState::COMPLETE => done += 1,
                _ => notdone += 1,
            }
        }
        return (done, notdone);
    }


    pub fn add_target(&mut self, filename: &str) {
        self.file_name = String::from(filename);
    }

    // TODO: Check to see if path is already set or not.
    pub fn set_base_path(&mut self, path: String) {
        self.paths.base = path.clone();
    }

    pub fn add_chunk(&mut self, record: scene_parse::Record) {
        println!("Adding chunk {}", record.scene_number);
        let newchunk = Chunk {
            scene_number: record.scene_number,
            start_frame: record.start_frame,
            end_frame: record.end_frame-1, // -1 because we don't want to end on the next chunk's start frame
            length_frames: record.length_frames-1, // because we're trimming 1 off the end frame
            state: EncodeState::NOT_STARTED,
            raw_file: String::new(),
            firstpass_file: String::new(),
            encoded_file: String::new(),
        };
        self.chunks.push(newchunk);
    }

    // Returns a chunk
    pub fn get_scene(&self, scene_num: usize) -> Option<Chunk> {

        for chunk in self.chunks.iter() {
            if chunk.scene_number == scene_num {
                return Some(chunk.clone());
            }
        }
        return None;
    }

    // TODO: when writing error behavior, this needs to return an error instead of panic
    pub fn update_chunk(&mut self, newchunk: Chunk) {

        for chunk in self.chunks.iter_mut() {
            if newchunk.scene_number == chunk.scene_number {
                println!("replacing chunk {}\n{:?}", chunk.scene_number, newchunk);
                *chunk = newchunk;
                return;
            }
        }
        // if we got here, it means that we didn't find our chunk
        panic!("We didn't find a chunk to update!");

    }

    pub fn save(&self) {
        let file_writer = fs::OpenOptions::new().write(true).truncate(true).create(true)
                                                .open(PROJECT_FILE).expect("problem opening writer");

        serde_yaml::to_writer(file_writer, &self).expect("problem serializing project");
    }

}