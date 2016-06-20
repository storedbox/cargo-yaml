use std::ascii::AsciiExt;
use std::cell::RefCell;
use std::fmt;
use std::io;
use log::{self, LogLevel, LogLevelFilter, LogMetadata, LogRecord};
use term::color::Color;
use term::{self, StderrTerminal, StdoutTerminal};

pub struct Logger {
    filtering_level: LogLevelFilter,
    stderr: RefCell<Box<StderrTerminal>>,
    stdout: RefCell<Box<StdoutTerminal>>,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Logger {
    fn with_filter(filtering_level: LogLevelFilter) -> term::Result<Self> {
        Ok(Logger {
            filtering_level: filtering_level,
            stderr: RefCell::new(try!(term::stderr().ok_or(term::Error::NotSupported))),
            stdout: RefCell::new(try!(term::stdout().ok_or(term::Error::NotSupported))),
        })
    }

    fn split_args(args: &fmt::Arguments) -> (String, String) {
        let args = format!("{}", args);
        let args = args.split(' ');
        let head = {
            let raw_head = args.clone().next().unwrap();
            let padding = (raw_head.len()..12).map(|_| ' ').collect::<String>();
            format!("{}{}", padding, raw_head)
        };
        let body = args.skip(1).fold(String::new(), |acc, s| format!("{} {}", acc, s));
        (head, body)
    }

    fn write_alarm(&self, record: &LogRecord, fgcolor: Color) -> io::Result<()> {
        let mut out = self.stderr.borrow_mut();
        let reset = out.supports_reset();

        if reset {
            // let _ = out.reset();
            let _ = out.fg(fgcolor);
            let _ = out.attr(term::Attr::Bold);
        }
        try!(write!(out,
                    "{}:",
                    format!("{}", record.level()).to_ascii_lowercase()));
        if reset {
            let _ = out.reset();
        }
        try!(writeln!(out, " {}", record.args()));
        out.flush()
    }

    fn write_info(&self, record: &LogRecord, fgcolor: Color) -> io::Result<()> {
        let (args_head, args_body) = Self::split_args(record.args());

        let mut out = self.stdout.borrow_mut();
        let reset = out.supports_reset();

        if reset {
            // let _ = out.reset();
            let _ = out.fg(fgcolor);
            let _ = out.attr(term::Attr::Bold);
        }
        try!(write!(out, "{}", args_head));
        if reset {
            let _ = out.reset();
        }
        try!(writeln!(out, "{}", args_body));
        out.flush()
    }

    fn write_debug(&self, record: &LogRecord, fgcolor: Color) -> io::Result<()> {
        let (args_head, args_body) = Self::split_args(record.args());

        let mut out = self.stdout.borrow_mut();
        let reset = out.supports_reset();

        if reset {
            // let _ = out.reset();
            let _ = out.fg(fgcolor);
            let _ = out.attr(term::Attr::Bold);
            let _ = out.attr(term::Attr::Dim);
        }
        try!(write!(out, "{}", args_head));
        if reset {
            let _ = out.reset();
        }
        try!(writeln!(out, "{}", args_body));
        out.flush()
    }

    fn write_trace(&self, record: &LogRecord, fgcolor: Color) -> io::Result<()> {
        let loc = record.location();
        let mut out = self.stderr.borrow_mut();
        let reset = out.supports_reset();

        if reset {
            // let _ = out.reset();
            let _ = out.fg(fgcolor);
            let _ = out.attr(term::Attr::Dim);
        }
        try!(write!(out, "[{}|", loc.module_path()));
        let _ = out.attr(term::Attr::Underline(true));
        try!(write!(out, "{}:{}", loc.file(), loc.line()));
        let _ = out.attr(term::Attr::Underline(false));
        try!(write!(out, "]"));
        if reset {
            let _ = out.reset();
        }
        try!(writeln!(out, " {}", record.args()));
        out.flush()
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.filtering_level
    }

    fn log(&self, record: &LogRecord) {
        use log::LogLevel::*;
        use term::color::*;

        if self.enabled(record.metadata()) {
            match record.level() {
                    Error => self.write_alarm(record, BRIGHT_RED),
                    Warn => self.write_alarm(record, BRIGHT_YELLOW),
                    Info => self.write_info(record, BRIGHT_GREEN),
                    Debug => self.write_debug(record, CYAN),
                    Trace => self.write_trace(record, MAGENTA),
                }
                .unwrap();
        }
    }
}

pub fn init(filtering_level: LogLevelFilter) -> Result<(), log::SetLoggerError> {
    let result_buffer = log::set_logger(|max_log_level| {
        max_log_level.set(filtering_level);
        Box::new(Logger::with_filter(filtering_level).unwrap())
    });
    trace!("logger initialized with filtering level '{}'",
           filtering_level);
    result_buffer
}
