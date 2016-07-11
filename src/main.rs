// allowed by default
#![allow(box_pointers)]
#![warn(fat_ptr_transmutes)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![allow(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![allow(unsafe_code)]
#![warn(unstable_features)]
#![allow(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

// lint-group "unused"
#![allow(dead_code)]
#![warn(path_statements)]
#![warn(unreachable_code)]
#![allow(unused_assignments)]
#![warn(unused_attributes)]
#![allow(unused_imports)]
#![warn(unused_must_use)]
#![warn(unused_mut)]
#![warn(unused_unsafe)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;

extern crate term;
extern crate toml;
extern crate yaml_rust;

use std::fs::File;
use std::io::{self, Read, Write};

use toml::Value as Toml;
use yaml_rust::YamlLoader;
use yaml_rust::yaml::Yaml;

mod logger;
mod opts;

use opts::{Options, Verbosity};

fn read_file(path: &str) -> io::Result<String> {
    debug!("Reading contents of `{}` into memory", path);
    File::open(path).and_then(|mut file| {
        let mut content = String::new();
        file.read_to_string(&mut content).map(|_| content)
    })
}

fn write_file(path: &str, content: &str) -> io::Result<()> {
    debug!("Writing {} characters to `{}`", content.len(), path);
    File::create(path).and_then(|mut file| file.write_all(content.as_bytes()).map(|_| ()))
}

fn map_yaml_to_toml(yaml: Yaml) -> Toml {
    trace!("Mapping template field `Yaml::{:?}`", yaml);
    match yaml.clone() {
        Yaml::String(s) => Toml::String(s),
        Yaml::Integer(i) => Toml::Integer(i),
        Yaml::Real(f) => Toml::Float(f.parse::<f64>().unwrap()),
        Yaml::Boolean(b) => Toml::Boolean(b),
        Yaml::Array(a) => Toml::Array(a.into_iter().map(map_yaml_to_toml).collect()),
        Yaml::Hash(h) => {
            Toml::Table(h.into_iter()
                .map(|(k, v)| (String::from(k.as_str().unwrap()), map_yaml_to_toml(v)))
                .collect())
        }
        Yaml::Alias(..) => {
            error!("YAML aliases are not supported");
            panic!()
        }
        Yaml::Null => Toml::Table(toml::Table::new()),
        Yaml::BadValue => {
            error!("Found malformed YAML construct");
            panic!()
        }
    }
}

fn process_template(path: &str) -> Yaml {
    let raw_yaml = read_file(path)
        .map_err(|err| {
            error!("Cannot read template file `{}`: {}", path, err);
            panic!()
        })
        .unwrap();
    debug!("Attempting deserialization of data as an YAML AST");
    YamlLoader::load_from_str(&raw_yaml).unwrap()[0].clone()
}

fn main() {
    let opts = Options::from_args();
    logger::init(match opts.verbosity {
        Verbosity::Normal => log::LogLevelFilter::Info,
        Verbosity::Verbose => log::LogLevelFilter::Debug,
        Verbosity::Quiet => log::LogLevelFilter::Error,
    }).unwrap();
    if !opts.show_usage {
        info!("Generating new Cargo manifest");
        let yaml = process_template(&opts.template_path);
        debug!("Mapping YAML constructs to TOML equivilents");
        let toml = map_yaml_to_toml(yaml);
        let raw_toml = format!("# Auto-generated from `{}`\n{}", opts.template_path, toml);
        debug!("Serializing TOML AST to plaintext TOML");
        write_file(&opts.manifest_path, &raw_toml)
            .map_err(|err| {
                error!("Cannot write to manifest file `{}`: {}",
                       opts.manifest_path,
                       err);
                panic!()
            })
            .unwrap();
    } else {
        println!("{}", include_str!("../usage.txt"));
    }
}
