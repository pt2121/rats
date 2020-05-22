#[macro_use]
extern crate lazy_static;
extern crate term_size;

use crate::parser::parse_log_line;
use crate::parser::parse_start_proc;
use crate::parser::{parse_death, LogLevel};
use crate::presenter::{Presenter, Printer};
use clap::{App, Arg};
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::str::FromStr;

mod parser;
mod presenter;

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("rats")
        .version("0.1")
        .author("pt2121@users.noreply.github.com")
        .arg(
            Arg::with_name("package")
                .short('p')
                .long("package")
                .value_name("applicationId")
                .multiple(true)
                .about("Application package name(s)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tag")
                .short('t')
                .long("tag")
                .value_name("TAG")
                .multiple(true)
                .about("Filter output by specified tag(s)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("level")
                .short('l')
                .long("level")
                .value_name("V,D,I,W,E,F,v,d,i,w,e,f")
                .about("Minimum level to be displayed")
                .takes_value(true),
        )
        .get_matches();

    let packages: Vec<&str> = matches.values_of("package").map_or(vec![], |v| v.collect());
    let tags: Vec<&str> = matches.values_of("tag").map_or(vec![], |v| v.collect());
    let level: Option<LogLevel> = matches
        .value_of("level")
        .and_then(|l| LogLevel::from_str(l).ok());

    let stdin = io::stdin();
    let presenter: Box<dyn Presenter> = Box::new(Printer::new());
    let mut pids: HashSet<String> = HashSet::new();
    let mut last_tag: Option<String> = None;

    for result in stdin.lock().lines() {
        let line = result?;
        if line.is_empty() {
            continue;
        }

        let log_line = parse_log_line(&line);
        if log_line.is_none() {
            continue;
        }

        let log = log_line.unwrap();

        if let Some(proc) = parse_start_proc(line.as_str())
            .filter(|proc| match_package(&packages, &proc.line_package))
        {
            pids.insert(proc.line_pid.clone());

            last_tag.take();
            presenter.print_proc_start(proc)
        }

        if let Some(proc) = parse_death(log.tag.as_str(), log.message.as_str())
            .filter(|proc| match_package(&packages, &proc.line_package))
        {
            pids.remove(&proc.line_pid);

            last_tag.take();
            presenter.print_proc_end(proc);
        }

        if !match_tag(&tags, &log.tag) {
            continue;
        }

        let new_tag = last_tag.clone().map_or(true, |t| t != log.tag);
        if new_tag {
            last_tag = Some(log.tag.clone());
        }

        if (packages.is_empty() || pids.contains(&log.owner))
            && level.map_or(true, |l| l <= log.level)
        {
            presenter.print_log(&log, new_tag)
        }
    }

    Ok(())
}

fn match_package(packages: &[&str], input: &str) -> bool {
    if packages.is_empty() {
        return true;
    }

    let vec: Vec<&str> = input.split(':').collect();
    let str = if vec.is_empty() { input } else { vec[0] };
    packages.contains(&str)
}

fn match_tag(tags: &[&str], tag: &str) -> bool {
    if tags.is_empty() {
        return true;
    }

    tags.contains(&tag)
}

#[cfg(test)]
mod tests {
    use crate::match_package;
    use crate::parser::LogLevel;
    use std::str::FromStr;

    #[test]
    fn match_package_basic() {
        let packages = vec!["com.test", "com.example"];
        assert!(match_package(&packages, "com.test:what"))
    }

    #[test]
    fn match_package_not_match() {
        let packages = vec!["com.test", "com.example"];
        assert_eq!(false, match_package(&packages, "com.meh:com.example"))
    }

    #[test]
    fn match_package_empty() {
        let packages = vec![];
        assert!(match_package(&packages, "com.meh:com.example"))
    }

    #[test]
    fn filter_log_level() {
        let error_level = LogLevel::from_str("E").unwrap();
        let warn_level = LogLevel::from_str("W").unwrap();

        assert!(error_level > warn_level)
    }
}
