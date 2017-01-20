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
    match metadata(&format!("../tracking/{}/",name)) {
        Ok(_) => panic!("That name is already in use"),
        Err(_) => {
            let git_path = format!("../tracking/{}/",name);
            let c1 = Command::new("mkdir").arg(&format!("../tracking/{}",name))
                    .output().expect("mkdir failed");
            let c2 = Command::new("git").args(&["-C",&git_path,"init"]).output()
                    .expect("git init failed");
        }
    }
}

fn commit_changes(name: &str) {
    //git add result.txt
    let add = Command::new("git").args(&["add",&format!("{}","blah")]).output()
            .expect("add failed");
    // git commit -m TIMESTAMP
    let comm = Command::new("git").args(&["commit","-m",&UTC::now().to_string()])
            .output().expect("commit failed");
}

fn get_changes(name: &str) -> Vec<u8> {
    let hashes = Command::new("git").args(&["log","-2","--pretty=format:\"%H\""])
                 .output().expect("git log failed");
    let commits: Vec<String> = String::from_utf8_lossy(&hashes.stdout)
                             .split_whitespace().map(|s| s.to_string())
                             .collect();
    let diffs = Command::new("git").args(&["diff",&commits[0],&commits[1]])
                .output().expect("failed to retrieve diff");
    diffs.stdout
}

fn fetch(name: &str) {
    let client = Client::new();
    let mut result = client.get("addr").send().unwrap();
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
        x => {
            let mut file = OpenOptions::new().write(true).append(true)
                        .create(true).open("fail-log").unwrap();
            let msg = format!("Attempt to fetch {} failed with status code {}","addr",x);
            file.write_all(msg.as_bytes());
        }
    }
}


fn main() {
    // mode name **args
    let inputs: Vec<String> = args().collect();
    match &*inputs[1] {
        "add" => {
            initialize(&inputs[2]);
            fetch(&inputs[2]);
        },
        "update" => {
            fetch(&inputs[2]);
            commit_changes(&inputs[2]);

        },
        "diffs" => {
            get_changes(&inputs[2]);
        },
        "names" => {},
        _ => println!("That mode not recognized.\nValid modes are 'add', 'update','diffs', or 'names'")
    }


}
