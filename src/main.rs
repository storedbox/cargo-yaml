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

#[derive(Clone, PartialEq, Debug)]
struct Options {
    color: Color,
    verbosity: Verbosity,
    manifest_path: String,
    template_path: String,
}

impl Options {
    fn new() -> Self {
        Options {
            color: Color::Auto,
            verbosity: Verbosity::Normal,
            manifest_path: "Cargo.toml".to_string(),
            template_path: "Cargo.yaml".to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color {
    Always,
    Auto,
    Never,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Verbosity {
    Normal,
    Verbose,
    Quiet,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum OptionsBuilderState {
    Ready,
    Finished,
    AwaitingColor,
    AwaitingManifestPath,
}

#[derive(Clone, PartialEq, Debug)]
struct OptionsBuilder {
    state: OptionsBuilderState,
    show_usage: bool,
    color: Option<Color>,
    verbosity: Option<Verbosity>,
    manifest_path: Option<String>,
    template_path: Option<String>,
}

macro_rules! try_set {
    ( $s:expr => $opt:expr => $val:expr ) => {
        if $opt.is_none() {
            $opt = Some($val);
        } else {
            $s.show_usage = true;
        }
    }
}

impl OptionsBuilder {
    pub fn new() -> Self {
        OptionsBuilder {
            state: OptionsBuilderState::Ready,
            show_usage: false,
            color: Default::default(),
            verbosity: Default::default(),
            manifest_path: Default::default(),
            template_path: Default::default(),
        }
    }

    pub fn push_arg(&mut self, arg: String) -> &mut Self {
        use OptionsBuilderState::*;
        match self.state {
            Finished => self.show_usage = true,
            Ready => self.push_new(arg),
            AwaitingColor => self.push_color(arg),
            AwaitingManifestPath => {
                self.state = OptionsBuilderState::Ready;
                try_set!(self => self.manifest_path => arg)
            }
        };
        self
    }

    fn push_new(&mut self, arg: String) {
        match &arg[..] {
            "-h" | "--help" => self.show_usage = true,
            "-v" | "--verbose" => try_set!(self => self.verbosity => Verbosity::Verbose),
            "-q" | "--quiet" => try_set!(self => self.verbosity => Verbosity::Quiet),
            "--manifest-path" => self.state = OptionsBuilderState::AwaitingManifestPath,
            "--color" => self.state = OptionsBuilderState::AwaitingColor,
            _ => {
                try_set!(self => self.template_path => arg);
                self.state = OptionsBuilderState::Finished;
            }
        };
    }

    fn push_color(&mut self, arg: String) {
        self.state = OptionsBuilderState::Ready;
        match &arg[..] {
            "always" => try_set!(self => self.color => Color::Always),
            "auto" => try_set!(self => self.color => Color::Auto),
            "never" => try_set!(self => self.color => Color::Never),
            _ => self.show_usage = true,
        };
    }

    pub fn build(self) -> Option<Options> {
        if self.show_usage {
            return None;
        }
        let mut opts = Options::new();
        if let Some(color) = self.color {
            opts.color = color;
        }
        if let Some(verbosity) = self.verbosity {
            opts.verbosity = verbosity;
        }
        if let Some(manifest_path) = self.manifest_path {
            opts.manifest_path = manifest_path;
        }
        if let Some(template_path) = self.template_path {
            opts.template_path = template_path;
        }
        Some(opts)
    }
}

fn main() {
    fn generate() {
        let raw_yaml = read_file("Cargo.yaml").expect("`Cargo.yaml` not found in working directory");
        println!("  Generating new `Cargo.toml` manifest file");
        let yaml = YamlLoader::load_from_str(&raw_yaml).unwrap()[0].clone();
        write_manifest("Cargo.toml", yaml_to_toml(yaml));
    }

    let mut opts = OptionsBuilder::new();
    for arg in env::args().skip(2) {
        opts.push_arg(arg);
    }

    if let Some(opts) = opts.build() {
        // do nothing
    } else {
        println!("{}", include_str!("../usage.txt"));
    }

    // println!("{:?}", env::args_os().collect::<Vec<_>>());
    // let args: Vec<_> = env::args_os().skip(2).collect();
    // if !args.is_empty() {
    //     let status = Command::new("cargo").args(&args).status().unwrap();
    //     process::exit(status.code().unwrap_or(status.signal().unwrap_or(0)));
    // }
}
