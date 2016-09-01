extern crate docopt;
extern crate rustc_serialize;
extern crate toml;
extern crate yaml_rust;

use std::io::prelude::*;

const MANIFEST: &'static str = include_str!("../Cargo.yaml");

const USAGE: &'static str = "
cargo-yaml
David Huffman <storedbox@outlook.com>
Generate a Cargo.toml manifest from an YAML template

Usage:
    cargo-yaml [options] [--] [<command> [<args>...]]

Options:
    -h, --help                Display this message
    -o, --manifest-path PATH  Output path for the generated manifest file
    -i, --template-path PATH  Path to the YAML template; can be `-` for stdin
    -V, --version             Print version info and exit
    -v, --verbose             Use verbose output
    -q, --quiet               No output printed to stdout
    -c, --color WHEN          Coloring: auto, always, never
";

mod opt {
    use std::fmt::Arguments as FmtArgs;
    use std::path::PathBuf;

    #[derive(Debug, RustcDecodable)]
    pub struct Args {
        arg_args: Vec<String>,
        arg_command: Option<String>,
        flag_manifest_path: Option<String>,
        pub flag_template_path: Option<String>,
        pub flag_color: Option<Color>,
        flag_quiet: bool,
        flag_verbose: bool,
        pub flag_version: bool,
    }

    impl Args {
        pub fn sub_argv(&self) -> Vec<String> {
            // TODO: Locate cargo's executable using our ppid rather than the PATH
            // TODO: When called by cargo what's up with all the cmdline args? Use argv[0]
            //       rather than the above method of location if possible
            if let Some(ref cmd) = self.arg_command {
                let mut argv = vec!["cargo".to_string(), cmd.clone()];
                argv.append(&mut self.arg_args.to_vec());
                argv
            } else {
                vec![]
            }
        }

        pub fn manifest_path(&self) -> PathBuf {
            PathBuf::from(self.flag_manifest_path
                .clone()
                .unwrap_or_else(|| "Cargo.toml".to_string()))
        }

        pub fn template_path(&self) -> Option<PathBuf> {
            if let Some(ref path) = self.flag_template_path {
                Some(PathBuf::from(path))
            } else {
                for name in &["Cargo.yaml", "Cargo.yml"] {
                    let path = PathBuf::from(name);
                    if path.as_path().exists() {
                        return Some(path);
                    }
                }
                None
            }
        }

        pub fn color(&self) -> Color {
            self.flag_color.clone().unwrap_or_default()
        }

        pub fn verbosity(&self) -> Verbosity {
            match (self.flag_quiet, self.flag_verbose) {
                (true, _) => Verbosity::Quiet,
                (false, false) => Verbosity::Normal,
                (_, true) => Verbosity::Verbose,
            }
        }
    }

    #[derive(Clone, Debug, RustcDecodable)]
    pub enum Color {
        Auto,
        Always,
        Never,
    }

    impl Color {
        pub fn is_enabled(&self) -> bool {
            use self::Color::*;
            match self {
                &Auto | &Always => true,
                &Never => false,
            }
        }
    }

    impl Default for Color {
        fn default() -> Self {
            Color::Auto
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Verbosity {
        Quiet,
        Normal,
        Verbose,
    }

    impl Verbosity {
        pub fn if_normal(&self, args: FmtArgs) {
            if self != &Verbosity::Quiet {
                println!("{}", args);
            }
        }

        pub fn if_verbose(&self, args: FmtArgs) {
            if self == &Verbosity::Verbose {
                println!("{}", args);
            }
        }
    }

    impl Default for Verbosity {
        fn default() -> Self {
            Verbosity::Normal
        }
    }
}

mod gen {
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::path::Path;
    use toml::Table as TomlTable;
    use toml::Value as Toml;
    use yaml_rust::{Yaml, YamlLoader};

    pub fn read_file(path: &Path) -> io::Result<String> {
        File::open(path).and_then(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content).map(|_| content)
        })
    }

    pub fn write_file(path: &Path, content: &str) -> io::Result<()> {
        File::create(path).and_then(|mut file| file.write_all(content.as_bytes()).map(|_| ()))
    }

    pub fn yaml_to_toml(yaml: Yaml) -> Toml {
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
            Yaml::Null => Toml::Table(TomlTable::new()),
            Yaml::BadValue => panic!(),
        }
    }

    pub fn process_template(path: &Path) -> Yaml {
        let raw_yaml = read_file(path)
            .map_err(|err| {
                panic!("cannot read from the given template path `{:?}`: {}",
                       path,
                       err)
            })
            .unwrap();
        YamlLoader::load_from_str(&raw_yaml).unwrap()[0].clone()
    }
}

fn version() -> String {
    use yaml_rust::{Yaml, YamlLoader};
    let docs = YamlLoader::load_from_str(MANIFEST).unwrap();
    docs[0]
        .as_hash()
        .and_then(|root| root.get(&Yaml::from_str("package")).and_then(|n| n.as_hash()))
        .and_then(|package| package.get(&Yaml::from_str("version")).and_then(|n| n.as_str()))
        .unwrap()
        .to_string()
}

// TODO: execute cargo subcommand upon completion (if provided)
fn main() {
    let args: opt::Args =
        docopt::Docopt::new(USAGE).expect("new(..) failed").decode().unwrap_or_else(|e| {
            println!("{}", e);
            std::process::exit(1);
        });
    if args.flag_version {
        println!("cargo-yaml v{}", version());
        return;
    }
    println!("{:?}", args);
    let manifest_path = args.manifest_path();
    let template_path = args.template_path();
    let verb = args.verbosity();
    if args.flag_color.is_some() {
        let _ = writeln!(std::io::stderr(),
                         "WARNING: the `--color` option is currently ignored");
    }

    verb.if_normal(format_args!("  Generating new Cargo manifest"));
    verb.if_verbose(format_args!("     Reading YAML from {:?}", template_path));
    let yaml = if let Some(ref path) = template_path {
        gen::process_template(path)
    } else {
        let mut stderr = std::io::stderr();
        let _ = writeln!(stderr,
                         "cargo-yaml: there is no file named 'Cargo.yaml' or 'Cargo.yml' in the \
                          current directory");
        let _ = writeln!(stderr, "Try 'cargo yaml --help' for more information.");
        std::process::exit(1);
    };
    let raw_toml = format!("# Auto-generated from {:?}\n{}",
                           template_path.unwrap(),
                           gen::yaml_to_toml(yaml));
    verb.if_verbose(format_args!("     Writing TOML to {:?}", manifest_path));
    gen::write_file(manifest_path.as_path(), &raw_toml)
        .map_err(|err| {
            panic!("cannot write to the given manifest output path {:?}: {}",
                   manifest_path,
                   err)
        })
        .unwrap();

    let sub_argv = args.sub_argv();
}
