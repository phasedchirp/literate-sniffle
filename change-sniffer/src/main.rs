extern crate hyper;
extern crate hyper_native_tls;
extern crate chrono;

use hyper::client::Client;
use hyper::status::StatusCode;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use chrono::UTC;

use std::io::{Read,Write};
use std::process::Command;
use std::env::args;
use std::fs::{OpenOptions,File};
use std::collections::HashMap;

fn setup() {
    Command::new("mkdir").arg("../tracking").output().expect("mkdir failed");
    Command::new("touch").arg("../config").output().expect("config creation failed");
    Command::new("touch").arg("../fail-log").output().expect("log creation failed");
}

fn read_config() -> HashMap<String,String> {
    let f = File::open("../config");
    let mut config = HashMap::new();
    match f {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            for s in contents.trim().lines() {
                let info: Vec<String> = s.split_whitespace()
                            .map(|x| x.to_string()).collect();
                config.insert(info[0].clone(),info[1].clone());
            }
        },
        Err(e) => {
            panic!("Failed to open config file: {:?}",e);
        }
    }
    config
}

fn update_config(name: &str, addr: &str) {
    let mut f = OpenOptions::new().write(true).append(true).create(true)
                .open("../config").unwrap();
    f.write_all(format!("{} {}\n",name,addr).as_bytes()).unwrap();
}

fn initialize(name: &str) {
    let git_path = format!("../tracking/{}/",name);
    let _ = Command::new("mkdir").arg(&git_path).output().expect("mkdir failed");
    let _ = Command::new("git").args(&["-C",&git_path,"init"]).output().expect("git init failed");
    let _ = Command::new("git").args(&["-C",&git_path,"config","user.name","change-sniffer"])
            .output().expect("set user.name failed");
    let _ = Command::new("git").args(&["-C",&git_path,"config","user.email",
            "change-sniffer@a.place"]).output().expect("set user.name failed");;
    let _ = Command::new("touch").arg(&(git_path + "result.txt")).output().expect("tracking file fail");

}

fn commit_changes(name: &str) -> String {
    //git add result.txt
    let git_path = format!("../tracking/{}/",name);
    let add = Command::new("git").args(&["-C",&git_path,"add","result.txt"]).output()
            .expect("add failed").status.code().unwrap();
    // git commit -m TIMESTAMP
    let comm = Command::new("git").args(&["-C",&git_path,"commit","-m",&UTC::now().to_string()])
            .output().expect("commit failed").status.code().unwrap();
    match (add,comm) {
        (0,0) => "successfully committed changes".to_string(),
        (0,_) => "git commit failed".to_string(),
        (_,0) => "git add failed".to_string(),
        (_,_) => "failed to update git repository".to_string()
    }
}

fn get_changes(name: &str) -> Vec<u8> {
    let dir = format!("../tracking/{}",name);
    let hashes = Command::new("git").args(&["-C",&dir,"log","-2","--pretty=format:\"%H\""])
                 .output().expect("git log failed");
    let commits: Vec<String> = String::from_utf8_lossy(&hashes.stdout)
                             .split_whitespace().map(|s| s.to_string())
                             .collect();
    println!("{}", commits[0]);
    println!("{}", commits[1]);
    let diffs = Command::new("git").args(&["diff",&commits[0],&commits[1]])
                .output().expect("failed to retrieve diff");
    diffs.stdout
}

fn fetch(name: &str, addr: &str) {
    let path = format!("../tracking/{}/result.txt",name);
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let mut result = client.get(addr).send().unwrap();
    match result.status {
        StatusCode::Ok => {
            let mut buff_old = String::new();
            {
                let mut file = OpenOptions::new().read(true).open(&path).unwrap();
                file.read_to_string(&mut buff_old).unwrap();
            }
            let mut buff_new = String::new();

            result.read_to_string(&mut buff_new).unwrap();
            // compare old and new responses, write if changes have occurred:
            if buff_old.trim() != buff_new.trim() {
                let mut file = OpenOptions::new().write(true).truncate(true).open(&path).unwrap();
                file.write_all(buff_new.trim().as_bytes()).unwrap();
                println!("Found difference");
                let res = commit_changes(name);
                println!("{}",res);
            }
        },
        x => {
            let mut file = OpenOptions::new().write(true).append(true)
                        .create(true).open("fail-log").unwrap();
            let msg = format!("Attempt to fetch {} failed with status code {}","addr",x);
            file.write_all(msg.as_bytes()).unwrap();
        }
    }
}


fn main() {
    // mode name **args
    let inputs: Vec<String> = args().collect();
    if inputs.len() == 1 {
        println!("The program takes the following options:\n");
        println!("setup -- generate tracking directory, config and log files");
        println!("add <name> <url> -- initialize tracking repo for <url>, with alias <name>");
        println!("update <name> -- update tracking repo for <name>");
        println!("diffs <name> -- get changes between two most recent versions of <name>");
        println!("names -- list currently assigned names and associated urls");
    }
    match &*inputs[1] {
        "setup" => setup(),
        "add" => {
            let config = read_config();
            match config.get(&inputs[2]) {
                Some(addr) => println!("The name {} is already in use for {}",&inputs[2],&addr),
                None => {
                    initialize(&inputs[2]);
                    fetch(&inputs[2],&inputs[3]);
                    update_config(&inputs[2],&inputs[3]);
                }
            }
        },
        "update" => {
            let config = read_config();
            match config.get(&inputs[2]) {
                Some(addr) => {
                    fetch(&inputs[2],&addr);
                },
                None => println!("That name isn't being tracked")
            }
        },
        "diffs" => {
            let msg = get_changes(&inputs[2]);
            println!("{:?}", &String::from_utf8_lossy(&msg));
        },
        "names" => {
            let config = read_config();
            for key in config.keys() {
                let val = config.get(key).unwrap();
                println!("{}: {}",key,val);
            }
        },
        _ => println!("That option was not recognized.\nValid modes are 'add', 'update','diffs', or 'names'")
    }


}
