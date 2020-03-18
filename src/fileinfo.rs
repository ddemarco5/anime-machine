use serde_json;
use std::fs::File;

use crate::commands;
use crate::commands::Operation;
use crate::project;


// Parses a given file and returns a pair, the container and first video codec Values, respectively
pub fn parse_container_and_codec(project: &project::Project, target: String) -> (serde_json::Value, serde_json::Value) {

    // Run our mediainfo command
    let mut inforunner = commands::make_mediainfo(project, target);
    let mut command = inforunner.prepare();

    command.output().expect("Failed to execute mediainfo process");

    // start parsing our file
    let file = File::open(inforunner.cleanup_files[1].clone()).expect("Error opening json file");

    let info: serde_json::Value = serde_json::from_reader(file).expect("Error parsing json file");

    let tracks = info.get("media").unwrap().get("track").unwrap();
    
    let containerinfo = tracks[0].clone();
    let codecinfo = tracks[1].clone();

    // Clean up from our mediainfo process
    inforunner.cleanup();

    return (containerinfo, codecinfo);

}

