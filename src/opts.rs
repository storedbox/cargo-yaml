use std::env;

#[derive(Clone, PartialEq, Debug)]
pub struct Options {
    pub color: Color,
    pub verbosity: Verbosity,
    pub manifest_path: String,
    pub template_path: String,
    pub show_usage: bool,
}

impl Options {
    pub fn new() -> Self {
        Options {
            color: Color::Always,
            verbosity: Verbosity::Normal,
            manifest_path: "Cargo.toml".to_string(),
            template_path: "Cargo.yaml".to_string(),
            show_usage: false,
        }
    }

    pub fn from_args() -> Self {
        env::args()
            .skip(2)
            .fold(OptionsBuffer::default(), |buf, arg| {
                match &arg[..] {
                    "-h" | "--help" => buf.set_show_usage(),
                    "-v" | "--verbose" => buf.set_verbosity(Verbosity::Verbose),
                    "-q" | "--quiet" => buf.set_verbosity(Verbosity::Quiet),
                    "--color" => buf.set_awaiting(OptionsBufferAwaiting::Color),
                    "--manifest-path" => buf.set_awaiting(OptionsBufferAwaiting::ManifestPath),
                    _ => buf.set_string(arg),
                }
            })
            .build()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Always,
    Never,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Verbosity {
    Normal,
    Verbose,
    Quiet,
}

#[derive(Clone, Copy, Debug)]
enum OptionsBufferAwaiting {
    Color,
    ManifestPath,
}

#[derive(Debug, Default)]
struct OptionsBuffer {
    color: Option<Color>,
    verbosity: Option<Verbosity>,
    manifest_path: Option<String>,
    template_path: Option<String>,
    show_usage: bool,
    awaiting: Option<OptionsBufferAwaiting>,
}

impl OptionsBuffer {
    fn build(self) -> Options {
        let mut new = Options::new();
        if let Some(color) = self.color {
            new.color = color;
        }
        if let Some(verbosity) = self.verbosity {
            new.verbosity = verbosity;
        }
        if let Some(manifest_path) = self.manifest_path {
            new.manifest_path = manifest_path;
        }
        if let Some(template_path) = self.template_path {
            new.template_path = template_path;
        }
        new.show_usage = self.show_usage;
        new
    }

    fn set<T>(usage: &mut bool, opt: &mut Option<T>, val: T) {
        if opt.is_none() {
            *opt = Some(val);
        } else {
            *usage = true;
        }
    }

    fn set_show_usage(mut self) -> Self {
        self.show_usage = true;
        self
    }

    fn set_verbosity(mut self, verbosity: Verbosity) -> Self {
        OptionsBuffer::set(&mut self.show_usage, &mut self.verbosity, verbosity);
        self
    }

    fn set_awaiting(mut self, state: OptionsBufferAwaiting) -> Self {
        OptionsBuffer::set(&mut self.show_usage, &mut self.awaiting, state);
        self
    }

    fn set_string(mut self, val: String) -> Self {
        match self.awaiting {
            Some(OptionsBufferAwaiting::Color) => {
                let color = match &val[..] {
                    "always" | "auto" => Color::Always,
                    "never" => Color::Never,
                    _ => panic!(),
                };
                self.awaiting = None;
                OptionsBuffer::set(&mut self.show_usage, &mut self.color, color);
                self
            }
            Some(OptionsBufferAwaiting::ManifestPath) => {
                self.awaiting = None;
                OptionsBuffer::set(&mut self.show_usage, &mut self.manifest_path, val);
                self
            }
            None => {
                OptionsBuffer::set(&mut self.show_usage, &mut self.template_path, val);
                self
            }
        }
    }
}
