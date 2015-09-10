// extern crate libc;
extern crate toml;
extern crate yaml_rust;

use std::env::args_os;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command,exit};
use std::str::FromStr;

// use libc::{c_char,c_int,execv};
use toml::Value as Toml;
use yaml_rust::{YamlLoader, yaml};
use yaml_rust::yaml::Yaml;

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

fn write_file<P: AsRef<Path>, S: Display>(path: P, content: &S) -> Option<()> {
    File::create(path).ok().and_then(|mut file| {
        file.write(b"# Auto-generated from `Cargo.yaml`\n").ok();
        file.write_fmt(format_args!("{}", content)).ok()
    })
}

fn yarray_to_tarray(yaml: &yaml::Array) -> toml::Array {
    let mut toml = toml::Array::new();
    for val in yaml.iter() {
        toml.push(yvalue_to_tvalue(val));
    }
    toml
}

fn yhash_to_ttable(yaml: &yaml::Hash) -> toml::Table {
    let mut toml = toml::Table::new();
    for (key, val) in yaml.iter() {
        toml.insert(String::from(key.as_str().unwrap()), yvalue_to_tvalue(val));
    }
    toml
}

fn yvalue_to_tvalue(yaml: &Yaml) -> Toml {
    match yaml {
        &Yaml::Real(ref float) => Toml::Float(f64::from_str(float).unwrap()),
        &Yaml::Integer(int) => Toml::Integer(int),
        &Yaml::String(ref string) => Toml::String(string.clone()),
        &Yaml::Boolean(bool) => Toml::Boolean(bool),
        &Yaml::Array(ref array) => Toml::Array(yarray_to_tarray(&array)),
        &Yaml::Hash(ref hash) => Toml::Table(yhash_to_ttable(&hash)),
        &Yaml::Alias(..) => unimplemented!(),
        &Yaml::Null => Toml::Table(toml::Table::new()),
        &Yaml::BadValue => panic!(),
    }
}

fn main() {
    let raw_yaml = read_file("Cargo.yaml").unwrap();
    let yaml = &YamlLoader::load_from_str(&raw_yaml).unwrap()[0];
    write_file("Cargo.toml", &yvalue_to_tvalue(&yaml));

    let args: Vec<_> = args_os().skip(2).collect();
    if !args.is_empty() {
        let status = Command::new("cargo").args(&args).status().unwrap();
        exit(status.code().unwrap_or(status.signal().unwrap_or(0)));
    }
}
