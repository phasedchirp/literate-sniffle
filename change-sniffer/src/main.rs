extern crate hyper;
extern crate chrono;

use hyper::client::{Client,Response};
use hyper::status::StatusCode;

use chrono::UTC;

use std::io::{Read,Write};
use std::process::Command;
use std::env::args;
use std::fs::{metadata,OpenOptions};


fn initialize(name: &str) {
    let git_path = format!("../tracking/{}/",name);
    let c1 = Command::new("mkdir").arg(&format!("../tracking/{}",name))
            .output().expect("mkdir failed");
    let c2 = Command::new("git").args(&["-C",&git_path,"init"]).output()
            .expect("git init failed");
}

fn commit_changes() {
    //git add result.txt
    let add = Command::new("git").args(&["add",&format!("{}","blah")]).output()
            .expect("add failed");
    // git commit -m TIMESTAMP
    let comm = Command::new("git").args(&["commit","-m",&UTC::now().to_string()])
            .output().expect("commit failed");
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
    match metadata(&format!("../tracking/{}/",inputs[1])) {
        Ok(_) => println!("Found git directory"),
        Err(_) => initialize(&inputs[1])
    }
    let client = Client::new();
    let mut result = client.get(&inputs[2]).send().unwrap();
    match result.status {
        StatusCode::Ok => {
            let mut buff_old = String::new();
            let mut buff_new = String::new();
            let mut file = OpenOptions::new().write(true).truncate(true)
                        .create(true).open("result.txt").unwrap();

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
