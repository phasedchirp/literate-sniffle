extern crate hyper;

use hyper::client::{Client,Response};
use hyper::status::StatusCode;

use std::io::{Read,Write};
use std::process::Command;
use std::env::args;
use std::fs::{metadata,OpenOptions};


fn initialize(name: &str) {
    let git_path = format!("../{}/",name);
    let c1 = Command::new("mkdir").arg(name).output().expect("mkdir failed");
    let c2 = Command::new("git").args(&["-C",&git_path,"init"]).output()
             .expect("git init failed");
}

fn get_changes() -> Vec<u8> {
    let hashes = Command::new("git").args(&["log","-2","--pretty=format:\"%H\""])
                 .output().expect("git log failed");
    let commits: Vec<String> = String::from_utf8_lossy(&hashes.stdout)
                             .split_whitespace().map(|s| s.to_string())
                             .collect();
    let diffs = Command::new("git").args(&["diff",&commits[0],&commits[1]])
                .output().expect("failed to retrieve diff");
    diffs.stdout
}


fn main() {
    let inputs: Vec<String> = args().collect(); // name, url, frequency
    match metadata("../.git/") {
        Ok(_) => println!("Found git directory"),
        Err(_) => println!("Didn't find git")
    }
    let client = Client::new();
    let mut result = client.get(&inputs[2]).send().unwrap();
    match result.status {
        StatusCode::Ok => {
            let mut buff_old = String::new();
            let mut buff_new = String::new();
            let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open("result.txt")
                        .unwrap();

            file.read_to_string(&mut buff_old);
            result.read_to_string(&mut buff_new);
            // compare old and new responses, write if changes have occurred:
            if buff_old != buff_new {
                file.write_all(buff_new.as_bytes());
            }
        },
        _ => println!("Some other status code")
    }
}
