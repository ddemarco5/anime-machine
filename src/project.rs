use serde::Serialize;
use serde::Deserialize;
use std::fs;
use std::path;

use crate::scene_parse;
use crate::commands;
use crate::dispatcher;

const PROJECT_FILE: &'static str = "project.yaml";

#[derive(Deserialize, Serialize)]
pub struct Project{
    pub file_name: String,
    pub paths: Dirs,
    pub chunks: Vec<Chunk>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Chunk {
    pub scene_number: usize,
    pub start_frame: usize,
    pub end_frame: usize,
    pub length_frames: usize,
    pub is_split: bool,
    pub raw_file: String,
    pub is_encoded: bool,
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

    pub fn new() -> Project {
        Project { 
            file_name: String::new(),
            paths: Dirs {
                base: String::new(),
                tools: String::from("tools"),
                tmp: String::from("tmp"),
                raw_chunks: String::from("raws"),
                encoded_chunks: String::new(),
            },
            chunks: Vec::new(),
        }
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
            is_split: false,
            raw_file: String::new(),
            is_encoded: false,
            encoded_file: String::new(),
        };
        self.chunks.push(newchunk);
    }

    // Returns a mutable reference to a chunk
    pub fn get_scene(&mut self, scene_num: usize) -> Option<&mut Chunk> {

        for chunk in self.chunks.iter_mut() {
            if chunk.scene_number == scene_num {
                return Some(chunk);
            }
        }

        return None;

    }

    pub fn from_file() -> Project {

        if !std::path::Path::new(PROJECT_FILE).exists() {
            println!("Project file doesn't exist! Giving you a brand new one instead.");
            return Project::new();
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

    pub fn save(&self) {
        let file_writer = fs::OpenOptions::new().write(true).truncate(true).create(true)
                                                .open(PROJECT_FILE).expect("problem opening writer");

        serde_yaml::to_writer(file_writer, &self).expect("problem serializing project");
    }

}