extern crate hyper;
extern crate chrono;

use hyper::client::{Client,Response};
use hyper::status::StatusCode;

use chrono::UTC;

use std::io::{Read,Write};
use std::process::Command;
use std::env::args;
use std::fs::{metadata,OpenOptions,File,DirBuilder};
use std::collections::HashMap;

fn setup() {
    Command::new("mkdir").arg("../tracking").output().expect("mkdir failed");
    Command::new("touch").arg("../.config").output().expect("config creation failed");
    Command::new("touch").arg("../fail-log").output().expect("log creation failed");
}

fn read_config() -> HashMap<String,String> {
    let f = File::open("../config");
    let mut config = HashMap::new();
    match f {
        Ok(mut file) => {

            let mut contents = String::new();
            file.read_to_string(&mut contents);
            let lines: Vec<&str> = contents.split("\n").collect();
            for s in lines {
                let info: Vec<String> = s.split_whitespace()
                            .map(|x| x.to_string()).collect();
                config.insert(info[0].clone(),info[1].clone());
            }
        },
        Err(e) => {
            // let mut config = HashMap::new();
            panic!("Failed to open config file: {:?}",e);
        }
    }
    config
}

fn update_config(name: &str, addr: &str) {
    let mut f = OpenOptions::new().write(true).append(true).create(true)
                .open("config").unwrap();
    f.write_all(format!("{} {}",name,addr).as_bytes());
}

fn initialize(name: &str, addr: &str) {
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

fn commit_changes(name: &str) -> String {
    //git add result.txt
    let add = Command::new("git").args(&["add",&format!("{}","blah")]).output()
            .expect("add failed").status.code().unwrap();
    // git commit -m TIMESTAMP
    let comm = Command::new("git").args(&["commit","-m",&UTC::now().to_string()])
            .output().expect("commit failed").status.code().unwrap();
    match (add,comm) {
        (0,0) => "success".to_string(),
        (0,_) => "commit failed".to_string(),
        (_,_) => "failure".to_string()
    }
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
        "setup" => setup(),
        "add" => {
            let mut config = read_config();
            match config.get(&inputs[2]) {
                Some(addr) => println!("The name {} is already in use for {}",&inputs[2],&addr),
                None => {
                    initialize(&inputs[2],&inputs[3]);
                    fetch(&inputs[2]);
                }
            }
        },
        "update" => {
            fetch(&inputs[2]);
            let res = commit_changes(&inputs[2]);
            println!("{}",res);
        },
        "diffs" => {
            get_changes(&inputs[2]);
        },
        "names" => {},
        _ => println!("That option was not recognized.\nValid modes are 'add', 'update','diffs', or 'names'")
    }


}
