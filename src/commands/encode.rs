use crate::commands;
use std::process;

// TODO: finish me
#[derive(Debug, Clone)]  
pub struct Encode {
    pub info: commands::JobInfo,
    pub crap: usize,
}

impl commands::Operation for Encode {
    
    /*
    fn execute(&self) {
        println!("encode command would execute here!");
    } */

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