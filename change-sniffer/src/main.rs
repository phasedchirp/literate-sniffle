extern crate hyper;

use hyper::client::Client;

use std::process::Command;
use std::env::args;

fn main() {
    let inputs: Vec<String> = args().collect(); // url, frequency
    let client = Client::new();
    let result = client.get(&inputs[1]).send().unwrap();
    println!("{:?}", result.status);
}
