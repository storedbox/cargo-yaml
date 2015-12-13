use std::env;

pub struct Options {
    pub color: Color,
    pub verbosity: Verbosity,
    pub manifest_path: String,
    pub template_path: String,
}

impl Options {
    pub fn new() -> Self {
        Options {
            color: Color::Auto,
            verbosity: Verbosity::Normal,
            manifest_path: "Cargo.toml".to_string(),
            template_path: "Cargo.yaml".to_string(),
        }
    }

    pub fn from_args() -> Option<Self> {
        let mut builder = OptionsBuilder::new();
        for arg in env::args().skip(2) {
            builder.push_arg(arg);
        }
        builder.build()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    Always,
    Auto,
    Never,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Verbosity {
    Normal,
    Verbose,
    Quiet,
}

#[derive(Clone, Copy, PartialEq)]
enum OptionsBuilderState {
    Ready,
    Finished,
    AwaitingColor,
    AwaitingManifestPath,
}

pub struct OptionsBuilder {
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

    pub fn build(self) -> Option<Options> {
        if self.show_usage || self.state == OptionsBuilderState::AwaitingColor || self.state == OptionsBuilderState::AwaitingManifestPath {
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

    pub fn push_arg(&mut self, arg: String) -> &mut Self {
        use self::OptionsBuilderState::*;
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
}
