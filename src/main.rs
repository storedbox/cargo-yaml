#![allow(dead_code)]

extern crate toml;
extern crate yaml_rust;

use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
// use std::os::unix::process::ExitStatusExt;
use std::path::Path;
// use std::process::{self, Command};

use toml::Value as Toml;
use yaml_rust::YamlLoader;
use yaml_rust::yaml::Yaml;

mod opts;

use self::opts::Options;

fn read_file<P: AsRef<Path>>(path: P) -> Option<String> {
    File::open(path).ok().and_then(|mut file| {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            Some(content)
        } else {
            None
        }
    })
}

fn write_manifest<P: AsRef<Path>, S: Display>(path: P, content: S) -> Option<()> {
    File::create(path).ok().and_then(|mut file| {
        file.write_fmt(format_args!("# Auto-generated from `Cargo.yaml`\n{}", content)).ok()
    })
}

fn yaml_to_toml(yaml: Yaml) -> Toml {
    match yaml {
        Yaml::String(s) => Toml::String(s),
        Yaml::Integer(i) => Toml::Integer(i),
        Yaml::Real(f) => Toml::Float(f.parse::<f64>().unwrap()),
        Yaml::Boolean(b) => Toml::Boolean(b),
        Yaml::Array(a) => Toml::Array(a.into_iter().map(yaml_to_toml).collect()),
        Yaml::Hash(h) => {
            Toml::Table(h.into_iter()
                         .map(|(k, v)| (String::from(k.as_str().unwrap()), yaml_to_toml(v)))
                         .collect())
        }
        Yaml::Alias(..) => unimplemented!(),
        Yaml::Null => Toml::Table(toml::Table::new()),
        Yaml::BadValue => panic!(),
    }
}

fn main() {
    fn generate() {
        let raw_yaml = read_file("Cargo.yaml").expect("`Cargo.yaml` not found in working directory");
        println!("  Generating new `Cargo.toml` manifest file");
        let yaml = YamlLoader::load_from_str(&raw_yaml).unwrap()[0].clone();
        write_manifest("Cargo.toml", yaml_to_toml(yaml));
    }
    let opts = Options::from_args();
    if let Some(_opts) = Options::from_args() {
        println!("no usage here");
    } else {
        println!("{}", include_str!("../usage.txt"));
    }
}
