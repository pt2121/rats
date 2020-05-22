use crate::parser::{LogLevel, LogLine, Process};
use ansi_term::Colour::White;
use ansi_term::{Colour, Style};

static TAG_WIDTH: usize = 32;
static HEADER_SIZE: usize = TAG_WIDTH + 1 + 3 + 1;
static WIDTH: usize = 180;

#[derive(Debug, Clone)]
struct PrinterError;

pub trait Presenter {
    fn print_proc_start(&self, process: Process);

    fn print_proc_end(&self, process: Process);

    fn print_log(&self, log: &LogLine, is_new_tag: bool);
}

pub struct Printer {
    colors: Colors,
}

impl Printer {
    pub fn new() -> Self {
        Printer {
            colors: Colors::new(),
        }
    }

    fn fmt_header(tag: &str, width: usize) -> String {
        format!("{tag:>0$}", width, tag = tag)
    }
}

impl Presenter for Printer {
    fn print_proc_start(&self, process: Process) {
        let message = format!(
            "Process {} ({}) created for {}",
            process.line_package,
            process.line_pid,
            process.target.unwrap_or_default()
        );
        let buf = indent_wrap(&message);
        println!("\n{}{}", Printer::fmt_header("", HEADER_SIZE), buf);
    }

    fn print_proc_end(&self, process: Process) {
        let message = format!(
            "Process {} ended for {}",
            process.line_pid, process.line_package
        );
        let buf = indent_wrap(&message);
        println!("\n{}{}", Printer::fmt_header("", HEADER_SIZE), buf);
    }

    fn print_log(&self, log: &LogLine, is_new_tag: bool) {
        // right-align tag title and allocate color if needed
        let display_tag = if is_new_tag {
            slice_from_end(&log.tag.as_str(), TAG_WIDTH - 1).unwrap_or(&log.tag)
        } else {
            ""
        };

        print!("{}", Printer::fmt_header(&display_tag, TAG_WIDTH));

        let style = match log.level {
            LogLevel::DEBUG => self.colors.debug,
            LogLevel::WARN => self.colors.warn,
            LogLevel::ERROR => self.colors.error,
            _ => White.dimmed().reverse(),
        };

        let level = style.paint(format!(" {} ", log.level)).to_string();
        let mut msg = String::new();
        msg.push_str(log.message.as_str());
        if let Some(date) = log.date.as_ref() {
            msg.push_str(" date=");
            msg.push_str(date.as_str());
        }

        if let Some(time) = log.time.as_ref() {
            msg.push_str(" time=");
            msg.push_str(time.as_str());
        }

        if let Some(tid) = log.tid.as_ref() {
            msg.push_str(" tid=");
            msg.push_str(tid.as_str());
        }
        let buf = indent_wrap(msg.as_str());
        println!(" {} {}", level, buf);
    }
}

#[derive(Debug, Default)]
pub struct Colors {
    pub debug: Style,
    pub warn: Style,
    pub error: Style,
}

impl Colors {
    fn new() -> Self {
        Colors {
            // https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
            debug: Colour::Fixed(111).bold().reverse(),
            warn: Colour::Fixed(222).bold().reverse(),
            error: Colour::Fixed(174).bold().reverse(),
        }
    }
}

fn indent_wrap(message: &str) -> String {
    let width = term_width().unwrap_or(WIDTH).min(WIDTH);
    let wrap_area = width - HEADER_SIZE;
    let mut current = 0;
    let mut buf = String::new();
    let chars = message.chars().collect::<Vec<_>>();
    while current < chars.len() {
        let next = chars.len().min(current + wrap_area);
        buf.push_str(
            chars[current..next]
                .iter()
                .clone()
                .collect::<String>()
                .as_ref(),
        );
        if next < chars.len() {
            buf.push('\n');
            buf.push_str(" ".repeat(HEADER_SIZE).as_str());
        }
        current = next
    }
    buf
}

fn term_width() -> Option<usize> {
    term_size::dimensions().map(|(w, _)| w)
}

fn slice_from_end(s: &str, n: usize) -> Option<&str> {
    s.char_indices().rev().nth(n).map(|(i, _)| &s[i..])
}
