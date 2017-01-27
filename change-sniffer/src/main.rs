extern crate hyper;
extern crate hyper_native_tls;
extern crate chrono;

use hyper::client::Client;
use hyper::status::StatusCode;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use chrono::UTC;

use std::io::{Read,Write,stdin};
use std::process::Command;
use std::env::{args,home_dir};
use std::fs::{OpenOptions,File,copy,canonicalize};
use std::collections::HashMap;

fn setup(home: &str) {
    let config_path = format!("{}/.sniffer-config",home);
    let mut dir = String::new();
    // Command::new("touch").arg("~/.sniffer-config").output().expect("config creation failed");
    println!("Please specify a directory for tracking repositories:");
    stdin().read_line(&mut dir).expect("failed to read input");
    let dir = dir.replace("~",home);
    let mut file = OpenOptions::new().write(true).truncate(true).create_new(true).open(config_path)
        .expect("failed to open config file");

    file.write_all(&dir.trim().as_bytes()).expect("failed to set tracking directory");
    let c_dir = Command::new("mkdir").arg(&dir.trim()).output()
        .expect("mkdir failed");
    if c_dir.status.code().unwrap() != 0 {
        println!("{}", String::from_utf8_lossy(&c_dir.stderr));
    }
    let _ = Command::new("mkdir").arg(&format!("{}/tracking",dir.trim())).output()
        .expect("mkdir failed");
    let _ = Command::new("touch").arg(&format!("{}/config",dir.trim())).output()
        .expect("tracking creation failed");
    let _ = Command::new("touch").arg(&format!("{}/fail-log",dir.trim())).output()
        .expect("log creation failed");
}

fn read_config(config: &str) -> HashMap<String,String> {
    println!("{}/config",config);
    let f = File::open(&format!("{}/config",config));
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

fn update_config(config: &str, name: &str, addr: &str) {
    let mut f = OpenOptions::new().write(true).append(true).create(true)
                .open(&format!("{}/config",config)).unwrap();
    f.write_all(format!("{} {}\n",name,addr).as_bytes()).unwrap();
}

fn initialize(config: &str, name: &str) {
    let dir = format!("{}/tracking/{}",config,name);
    let _ = Command::new("mkdir").arg(&dir).output().expect("mkdir failed");
    let _ = Command::new("git").args(&["-C",&dir,"init"]).output().expect("git init failed");
    let _ = Command::new("git").args(&["-C",&dir,"config","user.name","change-sniffer"])
            .output().expect("set user.name failed");
    let _ = Command::new("git").args(&["-C",&dir,"config","user.email",
            "change-sniffer@a.place"]).output().expect("set user.name failed");;
    let _ = Command::new("touch").arg(&(dir + "/result.txt")).output().expect("tracking file fail");

}

fn commit_changes(config: &str, name: &str) -> String {
    //git add result.txt
    let dir = format!("{}/tracking/{}",config,name);
    let add = Command::new("git").args(&["-C",&dir,"add","result.txt"]).output()
            .expect("add failed").status.code().unwrap();
    // git commit -m TIMESTAMP
    let comm = Command::new("git").args(&["-C",&dir,"commit","-m",&UTC::now().to_string()])
            .output().expect("commit failed").status.code().unwrap();
    match (add,comm) {
        (0,0) => "successfully committed changes".to_string(),
        (0,_) => "git commit failed".to_string(),
        (_,0) => "git add failed".to_string(),
        (_,_) => "failed to update git repository".to_string()
    }
}

fn get_changes(config: &str, name: &str) -> Vec<u8> {
    let dir = format!("{}/tracking/{}",config,name);
    // need to fix this bit
    // let hashes = Command::new("git").args(&["-C",&dir,"log","-2","--pretty=format:\"%H\""])
                //  .output().expect("git log failed");
    // let commits: Vec<String> = String::from_utf8_lossy(&hashes.stdout)
                            //  .split_whitespace().map(|s| s.to_string())
                            //  .collect();
    // println!("{}", commits[0]);
    // println!("{}", commits[1]);
    let diffs = Command::new("git").args(&["-C",&dir,"diff","HEAD^"])
                .output().expect("failed to retrieve diff");
    diffs.stdout
}

fn fetch(config: &str, name: &str, addr: &str) {
    println!("Checking {} for changes",addr);
    let path = format!("{}/tracking/{}/result.txt",config,name);
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    match client.get(addr).send() {
        Ok(mut result) => {
            match result.status {
                StatusCode::Ok => {
                    let mut buff_old = String::new();
                    {
                        let mut file = OpenOptions::new().read(true).open(&path).unwrap();
                        file.read_to_string(&mut buff_old).expect("something went wrong opening results file");
                    }
                    let mut buff_new = String::new();

                    result.read_to_string(&mut buff_new).unwrap();
                    // compare old and new responses, write if changes have occurred:
                    if buff_old.trim() != buff_new.trim() {
                        let mut file = OpenOptions::new().write(true).truncate(true).open(&path).unwrap();
                        file.write_all(buff_new.trim().as_bytes()).unwrap();
                        println!("{} differs from previously fetched version",addr);
                        let res = commit_changes(config,name);
                        println!("{}",res);
                    } else {
                        let msg = format!("{} was accessed at {} and no differences were found",
                            addr,&UTC::now().to_string());
                        println!("{}", msg);
                    }
                },
                x => {
                    let mut file = OpenOptions::new().write(true).append(true)
                                .create(true).open(&format!("{}/fail-log",config)).unwrap();
                    let msg = format!("Attempt to fetch {} failed with status code {}",addr,x);
                    file.write_all(msg.as_bytes()).unwrap();
                    println!("{}",msg);
                }
        }},
        Err(e) => {
            let mut file = OpenOptions::new().write(true).append(true)
                        .create(true).open("fail-log").unwrap();
            let msg = format!("Attempt to fetch {} failed with error {}",addr,e);
            file.write_all(msg.as_bytes()).unwrap();
            println!("{}",msg);
        }
    }
}


fn main() {
    // mode name **args
    let inputs: Vec<String> = args().collect();
    let mut tracking = String::new();
    let home = home_dir().unwrap().to_string_lossy().into_owned();
    let config_path = format!("{}/.sniffer-config",home);
    if (inputs.len() < 2) || (inputs[1] != "setup") {
        let mut sniffer_config = OpenOptions::new().read(true).open(&config_path)
            .expect("Failed to open configuration file ~/.sniffer-config");
        sniffer_config.read_to_string(&mut tracking);
    }

    if inputs.len() == 1 {
        println!("\nThis program takes one of the following options:");
        println!("setup <file>     -- generate tracking directory, configuration and log files.
                    Will use supplied file to populate the tracking list if
                    specified, otherwise generates an empty initial list.");
        println!("add <name> <url> -- initialize tracking repo for <url>, with alias <name>");
        println!("update <name>    -- update tracking repo for <name>");
        println!("all              -- update all currently tracked urls");
        println!("diffs <name>     -- get changes between two most recent versions of <name>");
        println!("names            -- list currently assigned names and associated urls\n");
    } else {
        match &*inputs[1] {
            "setup" => {
                setup(&home);
                if inputs.len() > 2 {
                    copy(&inputs[2],&format!("{}/config",home));
                    let config = read_config(&tracking);
                    for name in config.keys() {
                        initialize(&tracking,&name);
                        fetch(&tracking,&name,&config.get(name).unwrap());
                    }
                }
            },
            "add" => {
                let config = read_config(&tracking);
                match config.get(&inputs[2]) {
                    Some(addr) => println!("The name {} is already in use for {}",&inputs[2],&addr),
                    None => {
                        initialize(&tracking,&inputs[2]);
                        fetch(&tracking,&inputs[2],&inputs[3]);
                        update_config(&tracking,&inputs[2],&inputs[3]);
                    }
                }
            },
            "update" => {
                let config = read_config(&tracking);
                match config.get(&inputs[2]) {
                    Some(addr) => {
                        fetch(&tracking,&inputs[2],&addr);
                    },
                    None => println!("The name \"{}\" isn't associated with a tracking repo",&inputs[2])
                }
            },
            "all" => {
                let config = read_config(&tracking);
                for name in config.keys() {
                    fetch(&tracking,&name,&config.get(name).unwrap());
                }
            }
            "diffs" => {
                let msg = get_changes(&tracking,&inputs[2]);
                println!("{}", &String::from_utf8_lossy(&msg));
            },
            "names" => {
                let config = read_config(&tracking,);
                for key in config.keys() {
                    println!("{}: {}",key,&config.get(key).unwrap());
                }
            },
            _ => println!("That option was not recognized.\nValid modes are 'setup', 'add', 'update', 'all', 'diffs', or 'names'")
        }
    }
}
