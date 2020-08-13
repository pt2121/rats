use crate::parser::{LogLevel, LogLine, Process};
use ansi_term::Colour::White;
use ansi_term::{Colour, Style};

pub static DEFAULT_TAG_WIDTH: usize = 32;
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
    tag_width: usize,
    header_size: usize,
}

impl Printer {
    pub fn new(tag_width: usize) -> Self {
        Printer {
            colors: Colors::new(),
            tag_width,
            header_size: tag_width + 1 + 3 + 1,
        }
    }

    fn fmt_header(tag: &str, width: usize) -> String {
        format!("{tag:>0$}", width, tag = tag)
    }

    fn build_date_time_pid_str(
        log: &LogLine,
        is_new_tag: bool,
        msg: &mut String,
        tag_width: usize,
        level: &str,
    ) {
        if is_new_tag {
            if let Some(date) = log.date.as_ref() {
                msg.push_str("date=");
                msg.push_str(date.as_str());
                msg.push_str(" ");
            }

            if let Some(time) = log.time.as_ref() {
                msg.push_str("time=");
                msg.push_str(time.as_str());
                msg.push_str(" ");
            }

            if let Some(tid) = log.tid.as_ref() {
                msg.push_str("tid=");
                msg.push_str(tid.as_str());
                msg.push_str(" ");
            }

            if !msg.is_empty() {
                msg.remove(msg.len() - 1);
                msg.push('\n');
                msg.push_str(" ".repeat(tag_width + 1).as_str());
                msg.push_str(level);
                msg.push_str(" ");
            }
        }
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
        let buf = indent_wrap(&message, term_width_or_width(WIDTH), self.header_size);
        println!("\n{}{}", Printer::fmt_header("", self.header_size), buf);
    }

    fn print_proc_end(&self, process: Process) {
        let message = format!(
            "Process {} ended for {}",
            process.line_pid, process.line_package
        );
        let buf = indent_wrap(&message, term_width_or_width(WIDTH), self.header_size);
        println!("\n{}{}", Printer::fmt_header("", self.header_size), buf);
    }

    fn print_log(&self, log: &LogLine, is_new_tag: bool) {
        let display_tag = if is_new_tag {
            take_last(&log.tag.as_str(), self.tag_width).unwrap_or(&log.tag)
        } else {
            ""
        };

        print!("{}", Printer::fmt_header(&display_tag, self.tag_width));

        let style = match log.level {
            LogLevel::DEBUG => self.colors.debug,
            LogLevel::WARN => self.colors.warn,
            LogLevel::ERROR => self.colors.error,
            _ => White.dimmed().reverse(),
        };

        let level = style.paint(format!(" {} ", log.level)).to_string();
        let mut msg = String::new();
        Printer::build_date_time_pid_str(log, is_new_tag, &mut msg, self.tag_width, level.as_str());
        let buf = indent_wrap(
            log.message.as_str(),
            term_width_or_width(WIDTH),
            self.header_size,
        );
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

fn indent_wrap(message: &str, width: usize, header_size: usize) -> String {
    let wrap_area = width - header_size;
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
            buf.push_str(" ".repeat(header_size).as_str());
        }
        current = next
    }
    buf
}

fn term_width_or_width(width: usize) -> usize {
    term_width().unwrap_or(width).min(width)
}

fn term_width() -> Option<usize> {
    term_size::dimensions().map(|(w, _)| w)
}

fn take_last(s: &str, size: usize) -> Option<&str> {
    if size < 1 {
        return None;
    }
    if size >= s.len() {
        return Some(s);
    }
    s.char_indices().rev().nth(size - 1).map(|(i, _)| &s[i..])
}

#[cfg(test)]
mod tests {
    use crate::parser::{LogLevel, LogLine};
    use crate::presenter::{indent_wrap, take_last, Printer, DEFAULT_TAG_WIDTH};

    static HEADER_SIZE: usize = DEFAULT_TAG_WIDTH + 1 + 3 + 1;

    #[test]
    fn test_fmt_header_basic() {
        let formatted = Printer::fmt_header("TAG", 4);

        assert_eq!(formatted, " TAG")
    }

    #[test]
    fn test_fmt_header_no_filled() {
        let formatted = Printer::fmt_header("BANGKOK", 4);

        assert_eq!(formatted, "BANGKOK")
    }

    #[test]
    fn test_take_last_basic() {
        let sliced = take_last("54321", 2);

        assert_eq!(sliced.unwrap(), "21")
    }

    #[test]
    fn test_take_last_short() {
        let sliced = take_last("1", 2);

        assert_eq!(sliced.unwrap(), "1")
    }

    #[test]
    fn test_take_last_invalid_size() {
        let sliced = take_last("54321", 0);

        assert!(sliced.is_none())
    }

    #[test]
    fn test_indent_wrap_short() {
        let result = indent_wrap("01234", HEADER_SIZE + 5, HEADER_SIZE);

        assert_eq!("01234", result)
    }

    #[test]
    fn test_indent_wrap_long() {
        let result = indent_wrap("0123456789", HEADER_SIZE + 5, HEADER_SIZE);

        assert_eq!("01234\n                                     56789", result)
    }

    #[test]
    fn add_date_time_pid() {
        let line = LogLine {
            level: LogLevel::VERBOSE,
            tag: "tag".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            date: Some("date".to_string()),
            time: Some("time".to_string()),
            tid: Some("tid".to_string()),
        };

        let mut msg = String::new();
        Printer::build_date_time_pid_str(&line, true, &mut msg, HEADER_SIZE + 5, "V");

        assert_eq!(
            "date=date time=time tid=tid\n                                           V ",
            msg
        )
    }

    #[test]
    fn not_add_date_time_pid_if_old_tag() {
        let line = LogLine {
            level: LogLevel::VERBOSE,
            tag: "tag".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            date: Some("date".to_string()),
            time: Some("time".to_string()),
            tid: Some("tid".to_string()),
        };

        let mut msg = String::new();
        Printer::build_date_time_pid_str(&line, false, &mut msg, HEADER_SIZE + 5, "V");

        assert_eq!("", msg)
    }

    #[test]
    fn not_add_date_time_pid_if_none() {
        let line = LogLine {
            level: LogLevel::VERBOSE,
            tag: "tag".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            date: None,
            time: None,
            tid: None,
        };

        let mut msg = String::new();
        Printer::build_date_time_pid_str(&line, false, &mut msg, HEADER_SIZE + 5, "V");

        assert_eq!("", msg)
    }

    #[test]
    fn add_date_time_pid_header_tag_width_0() {
        let line = LogLine {
            level: LogLevel::VERBOSE,
            tag: "tag".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            date: Some("date".to_string()),
            time: Some("time".to_string()),
            tid: Some("tid".to_string()),
        };

        let mut msg = String::new();
        Printer::build_date_time_pid_str(&line, true, &mut msg, 0, "V");

        assert_eq!("date=date time=time tid=tid\n V ", msg)
    }

    #[test]
    fn add_date_time_pid_header_e_level() {
        let line = LogLine {
            level: LogLevel::VERBOSE,
            tag: "tag".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            date: Some("date".to_string()),
            time: Some("time".to_string()),
            tid: Some("tid".to_string()),
        };

        let mut msg = String::new();
        Printer::build_date_time_pid_str(&line, true, &mut msg, 0, "E");

        assert_eq!("date=date time=time tid=tid\n E ", msg)
    }

    #[test]
    fn new_tag_width() {
        let printer = Printer::new(50);

        assert_eq!(55, printer.header_size);
        assert_eq!(50, printer.tag_width)
    }
}
