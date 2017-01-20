extern crate hyper;

use hyper::client::{Client,Response};
use hyper::status::StatusCode;

use std::io::{Read,Write};
use std::process::Command;
use std::env::args;
use std::fs::{metadata,OpenOptions};

fn main() {
    let inputs: Vec<String> = args().collect(); // url, frequency
    match metadata("../.git/") {
        Ok(_) => println!("Found git directory"),
        Err(_) => println!("Didn't find git")
    }
    let client = Client::new();
    let mut result = client.get(&inputs[1]).send().unwrap();
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
