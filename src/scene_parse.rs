use std::error::Error;
use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Record {
    //We need all these preprocessing shit for the automatic deserialize
    #[serde(rename="Scene Number")]
    pub scene_number: usize,
    #[serde(rename="Start Frame")]
    pub start_frame: usize,
    #[serde(rename="Start Timecode")]
    start_timecode: String,
    #[serde(rename="Start Time (seconds)")]
    start_time: f64,
    #[serde(rename="End Frame")]
    pub end_frame: usize,
    #[serde(rename="End Timecode")]
    end_timecode: String,
    #[serde(rename="End Time (seconds)")]
    end_time: f64,
    #[serde(rename="Length (frames)")]
    pub length_frames: usize,
    #[serde(rename="Length (timecode)")]
    length_timecode: String,
    #[serde(rename="Length (seconds)")]
    length_seconds: f64,
}

// Takes a vector passed to it, and will fill it with scenes.
pub fn build_records(filename: &str) -> Result<Vec<Record>, Box<dyn Error>> {

    // build our reader
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(filename).expect("Problem creating csv reader");

    let mut csv_data: Vec<Record> = Vec::new();
    // populate our vec
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: Record = result.expect("Error reading result");
        csv_data.push(record);
    }
    Ok(csv_data)

}