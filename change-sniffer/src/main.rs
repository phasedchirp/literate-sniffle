extern crate hyper;

use hyper::client::{Client,Response};
// use hyper::client::response::Response;
use hyper::status::StatusCode;

use std::io::Read;
use std::process::Command;
use std::env::args;

fn main() {
    let inputs: Vec<String> = args().collect(); // url, frequency
    let client = Client::new();
    let mut result = client.get(&inputs[1]).send().unwrap();
    match result.status {
        StatusCode::Ok => {
            let mut buf = String::new();
            result.read_to_string(&mut buf);
            println!("{:?}", buf);
        },
        _ => println!("Some other status code")
    }
}
