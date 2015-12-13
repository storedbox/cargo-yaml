extern crate toml;
extern crate yaml_rust;

use std::fs::File;
use std::io::{self, Read, Write};

use toml::Value as Toml;
use yaml_rust::YamlLoader;
use yaml_rust::yaml::Yaml;

mod opts;

use opts::{Options, Verbosity};

fn read_file(path: &str) -> io::Result<String> {
    File::open(path).and_then(|mut file| {
        let mut content = String::new();
        file.read_to_string(&mut content).map(|_| content)
    })
}

fn write_file(path: &str, content: &str) -> io::Result<()> {
    File::create(path).and_then(|mut file| file.write_all(content.as_bytes()).map(|_| ()))
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

fn process_template(path: &str) -> Yaml {
    let raw_yaml = read_file(path)
                       .map_err(|err| {
                           panic!("cannot read from the given template path `{}`: {}",
                                  path,
                                  err)
                       })
                       .unwrap();
    YamlLoader::load_from_str(&raw_yaml).unwrap()[0].clone()
}

fn main() {
    let opts = Options::from_args();
    if !opts.show_usage {
        if opts.verbosity != Verbosity::Quiet {
            println!("  Generating new Cargo manifest");
        }
        if opts.verbosity == Verbosity::Verbose {
            println!("     Reading YAML from `{}`", opts.template_path)
        }
        let yaml = process_template(&opts.template_path);
        let toml = yaml_to_toml(yaml);
        let raw_toml = format!("# Auto-generated from `{}`\n{}", opts.template_path, toml);
        if opts.verbosity == Verbosity::Verbose {
            println!("     Writing TOML to `{}`", opts.manifest_path);
        }
        write_file(&opts.manifest_path, &raw_toml)
            .map_err(|err| {
                panic!("cannot write to the given manifest output path `{}`: {}",
                       opts.manifest_path,
                       err)
            })
            .unwrap();
    } else {
        println!("{}", include_str!("../usage.txt"));
    }
}
