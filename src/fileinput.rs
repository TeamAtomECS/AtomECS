use std::fs::File;
use std::io::prelude::*;
use yaml_rust::yaml::{Hash, Yaml};
use yaml_rust::YamlLoader;

fn main() {
    println!("Yaml_input");
    let file = "inputparameter.yaml";
    load_file(file);
}

fn load_file(file: &str) {
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let docs = YamlLoader::load_from_str(&contents).unwrap();

    // iterate / process doc[s] ..
}