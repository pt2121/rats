use core::fmt;
use regex::Regex;
use std::fmt::Formatter;
use std::str::FromStr;

lazy_static! {
    // -v brief
    // "E/GnssHAL_GnssInterface( 1800): gnssSvStatusCb: b: input svInfo.flags is 8"
    static ref LOG_LINE_BRIEF: Regex = Regex::new(r"^(?P<level>[A-Z])/(?P<tag>.+?)\( *(?P<owner>\d+)\): (?P<message>.*?)$").unwrap();

    // 05-19 06:57:59.912  2045  2140 W AppOps  : Noting op not finished: uid 10102 pkg com.google.android.gms code 41 time=1589896674895 duration=0
    static ref LOG_LINE: Regex = Regex::new(r"^(?P<date>\d\d-\d\d)\s(?P<time>\d\d:\d\d:\d\d\.\d\d\d)\s+(?P<owner>\d+)\s+(?P<tid>\d+)\s+(?P<level>[A-Z])\s+(?P<tag>.+?)\s*: (?P<message>.*?)$").unwrap();

    // I/ActivityManager( 2045): Start proc 10212:com.google.android.gms.ui/u0a102 for service {com.google.android.gms/com.google.android.gms.chimera.UiIntentOperationService}
    static ref PID_START_5_1: Regex = Regex::new(r"^.*: Start proc (?P<line_pid>\d+):(?P<line_package>[a-zA-Z0-9._:]+)/[a-z0-9]+ for (?P<target>.*)$").unwrap();

    // I/ActivityManager( 2045): Killing 8822:com.google.android.apps.maps/u0a120 (adj 985): empty for 2733s
    static ref PID_KILL: Regex = Regex::new(r"^Killing (?P<pid>\d+):(?P<package_line>[a-zA-Z0-9._:]+)/[^:]+: (.*)$").unwrap();

    static ref PID_LEAVE: Regex = Regex::new(r"^No longer want (?P<package_line>[a-zA-Z0-9._:]+) \(pid (?P<pid>\d+)\): .*$").unwrap();

    // I/ActivityManager( 2045): Process com.example.test (pid 7404) has died: vis+99 TOP
    static ref PID_DEATH: Regex = Regex::new(r"^Process (?P<package_line>[a-zA-Z0-9._:]+) \(pid (?P<pid>\d+)\) has died.?$").unwrap();

    // static ref BACKTRACE_LINE: Regex = Regex::new(r"^#(.*?)pc\s(.*?)$").unwrap();
}

pub struct Process {
    pub line_pid: String,
    pub line_package: String,
    pub target: Option<String>,
}

pub struct LogLine {
    pub level: LogLevel,
    pub tag: String,
    pub owner: String,
    pub message: String,
    pub date: Option<String>,
    pub time: Option<String>,
    pub tid: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    VERBOSE = 0,
    DEBUG,
    INFO,
    WARN,
    ERROR,
    ASSERT,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::VERBOSE => write!(f, "V"),
            LogLevel::DEBUG => write!(f, "D"),
            LogLevel::INFO => write!(f, "I"),
            LogLevel::WARN => write!(f, "W"),
            LogLevel::ERROR => write!(f, "E"),
            LogLevel::ASSERT => write!(f, "A"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseLogLevelError {
    UnknownLogLevel,
}

impl FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "V" | "v" => Ok(LogLevel::VERBOSE),
            "D" | "d" => Ok(LogLevel::DEBUG),
            "I" | "i" => Ok(LogLevel::INFO),
            "W" | "w" => Ok(LogLevel::WARN),
            "E" | "e" => Ok(LogLevel::ERROR),
            "A" | "a" => Ok(LogLevel::ASSERT),
            _ => Err(ParseLogLevelError::UnknownLogLevel),
        }
    }
}

pub fn parse_start_proc(line: &str) -> Option<Process> {
    PID_START_5_1.captures(line).and_then(|caps| {
        let pid = caps.name("line_pid");
        let package = caps.name("line_package");
        let target = caps.name("target").map(|s| s.as_str().to_string());
        if pid.is_some() && package.is_some() {
            let id = pid.map(|s| s.as_str().to_string()).unwrap();
            Some(Process {
                line_pid: id,
                line_package: package.map(|s| s.as_str().to_string()).unwrap(),
                target,
            })
        } else {
            None
        }
    })
}

pub fn parse_death(tag: &str, message: &str) -> Option<Process> {
    if tag != "ActivityManager" {
        return None;
    }

    pid_kill(message)
        .or_else(|| pid_leave(message))
        .or_else(|| pid_death(message))
}

fn pid_leave(message: &str) -> Option<Process> {
    PID_LEAVE.captures(message).and_then(|caps| {
        let pid = caps.name("pid");
        let package = caps.name("package_line");
        if pid.is_some() && package.is_some() {
            Some(Process {
                line_pid: pid.map(|s| s.as_str().to_string()).unwrap(),
                line_package: package.map(|s| s.as_str().to_string()).unwrap(),
                target: None,
            })
        } else {
            None
        }
    })
}

fn pid_death(message: &str) -> Option<Process> {
    PID_DEATH.captures(message).and_then(|caps| {
        let pid = caps.name("pid");
        let package = caps.name("package_line");
        if pid.is_some() && package.is_some() {
            Some(Process {
                line_pid: pid.map(|s| s.as_str().to_string()).unwrap(),
                line_package: package.map(|s| s.as_str().to_string()).unwrap(),
                target: None,
            })
        } else {
            None
        }
    })
}

fn pid_kill(message: &str) -> Option<Process> {
    PID_KILL.captures(message).and_then(|caps| {
        let pid = caps.name("pid");
        let package = caps.name("package_line");
        if pid.is_some() && package.is_some() {
            Some(Process {
                line_pid: pid.map(|s| s.as_str().to_string()).unwrap(),
                line_package: package.map(|s| s.as_str().to_string()).unwrap(),
                target: None,
            })
        } else {
            None
        }
    })
}

pub fn parse_log_line(line: &str) -> Option<LogLine> {
    log_line(line).or_else(|| log_line_brief(line))
}

fn log_line(line: &str) -> Option<LogLine> {
    LOG_LINE.captures(&line).map(|caps| {
        let level: LogLevel = caps.name("level").map_or(LogLevel::VERBOSE, |s| {
            LogLevel::from_str(s.as_str()).unwrap_or(LogLevel::VERBOSE)
        });
        let tag = caps
            .name("tag")
            .map_or(String::new(), |s| s.as_str().trim().to_string());
        let owner = caps
            .name("owner")
            .map_or(String::new(), |s| s.as_str().to_string());
        let message = caps
            .name("message")
            .map_or(String::new(), |s| s.as_str().to_string());
        let date = caps.name("date").map(|s| s.as_str().to_string());
        let time = caps.name("time").map(|s| s.as_str().to_string());
        let tid = caps.name("tid").map(|s| s.as_str().to_string());
        LogLine {
            level,
            tag,
            owner,
            message,
            date,
            time,
            tid,
        }
    })
}

fn log_line_brief(line: &str) -> Option<LogLine> {
    LOG_LINE_BRIEF.captures(&line).map(|caps| {
        let level: LogLevel = caps.name("level").map_or(LogLevel::VERBOSE, |s| {
            LogLevel::from_str(s.as_str()).unwrap_or(LogLevel::VERBOSE)
        });
        let tag = caps
            .name("tag")
            .map_or(String::new(), |s| s.as_str().trim().to_string());
        let owner = caps
            .name("owner")
            .map_or(String::new(), |s| s.as_str().to_string());
        let message = caps
            .name("message")
            .map_or(String::new(), |s| s.as_str().to_string());
        LogLine {
            level,
            tag,
            owner,
            message,
            date: None,
            time: None,
            tid: None,
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::LOG_LINE;
    use crate::parser::PID_DEATH;
    use crate::parser::PID_KILL;
    use crate::parser::PID_START_5_1;
    use crate::parser::{parse_death, parse_log_line, parse_start_proc, LogLevel};

    #[test]
    fn test_parse_start_proc() {
        let str_log: &str = "I/ActivityManager( 2045): Start proc 10212:com.google.android.gms.ui/u0a102 for service {com.google.android.gms/com.google.android.gms.chimera.UiIntentOperationService}";
        let proc = parse_start_proc(str_log).unwrap();
        assert_eq!(proc.line_pid, "10212");
        assert_eq!(proc.line_package, "com.google.android.gms.ui");
        assert_eq!(proc.target.unwrap(), "service {com.google.android.gms/com.google.android.gms.chimera.UiIntentOperationService}");
    }

    #[test]
    fn test_parse_start_proc_missing_pid() {
        let str_log: &str = "I/ActivityManager( 2045): Start proc :com.google.android.gms.ui/u0a102 for service {com.google.android.gms/com.google.android.gms.chimera.UiIntentOperationService}";
        let proc = parse_start_proc(str_log);
        assert!(proc.is_none());
    }

    #[test]
    fn test_parse_death() {
        let str_log: &str = "Process com.example.urg (pid 7404) has died";
        let proc = parse_death("ActivityManager", str_log).unwrap();
        assert_eq!(proc.line_pid, "7404");
        assert_eq!(proc.line_package, "com.example.urg");
    }

    #[test]
    fn test_parse_death_missing_pid() {
        let str_log: &str = "Process com.example.urg (pid ) has died";
        let proc = parse_death("ActivityManager", str_log);
        assert!(proc.is_none());
    }

    #[test]
    fn test_parse_log_line() {
        let str_log: &str = "05-19 06:57:59.912  2045  2140 W AppOps  : Noting op not finished: uid 10102 pkg com.google.android.gms code 41 time=1589896674895 duration=0";

        let log = parse_log_line(str_log).unwrap();

        assert_eq!(log.level, LogLevel::WARN);
        assert_eq!(log.tag, "AppOps");
        assert_eq!(log.tid.unwrap().as_str(), "2140");
        assert_eq!(log.owner, "2045");
        assert_eq!(log.date.unwrap().as_str(), "05-19");
        assert_eq!(log.time.unwrap().as_str(), "06:57:59.912");
        assert_eq!(log.message, "Noting op not finished: uid 10102 pkg com.google.android.gms code 41 time=1589896674895 duration=0");
    }

    #[test]
    fn test_parse_log_line_brief() {
        let str_log: &str =
            "E/GnssHAL_GnssInterface( 1800): gnssSvStatusCb: b: input svInfo.flags is 8";

        let log = parse_log_line(str_log).unwrap();

        assert_eq!(log.level, LogLevel::ERROR);
        assert_eq!(log.tag, "GnssHAL_GnssInterface");
        assert!(log.tid.is_none());
        assert_eq!(log.owner, "1800");
        assert!(log.date.is_none());
        assert!(log.time.is_none());
        assert_eq!(log.message, "gnssSvStatusCb: b: input svInfo.flags is 8");
    }

    #[test]
    fn regex_log_line() {
        let str_log: &str = "05-19 06:57:59.912  2045  2140 W AppOps  : Noting op not finished: uid 10102 pkg com.google.android.gms code 41 time=1589896674895 duration=0";

        let caps = LOG_LINE.captures(str_log).unwrap();
        let date = caps.name("date").map_or("", |s| s.as_str());
        let time = caps.name("time").map_or("", |s| s.as_str());
        let owner = caps.name("owner").map_or("", |s| s.as_str());
        let tid = caps.name("tid").map_or("", |s| s.as_str());
        let level = caps.name("level").map_or("", |s| s.as_str());
        let tag = caps.name("tag").map_or("", |s| s.as_str().trim());
        let message = caps.name("message").map_or("", |s| s.as_str().trim());

        assert_eq!(date, "05-19");
        assert_eq!(time, "06:57:59.912");
        assert_eq!(owner, "2045");
        assert_eq!(tid, "2140");
        assert_eq!(level, "W");
        assert_eq!(tag, "AppOps");
        assert_eq!(message, "Noting op not finished: uid 10102 pkg com.google.android.gms code 41 time=1589896674895 duration=0")
    }

    #[test]
    fn regex_log_line_2() {
        let str_log: &str = "05-19 06:57:55.890  1800  2437 E GnssHAL_GnssInterface: gnssSvStatusCb: a: input svInfo.flags is 8";

        let caps = LOG_LINE.captures(str_log).unwrap();
        let date = caps.name("date").map_or("", |s| s.as_str());
        let time = caps.name("time").map_or("", |s| s.as_str());
        let owner = caps.name("owner").map_or("", |s| s.as_str());
        let tid = caps.name("tid").map_or("", |s| s.as_str());
        let level = caps.name("level").map_or("", |s| s.as_str());
        let tag = caps.name("tag").map_or("", |s| s.as_str().trim());
        let message = caps.name("message").map_or("", |s| s.as_str().trim());

        assert_eq!(date, "05-19");
        assert_eq!(time, "06:57:55.890");
        assert_eq!(owner, "1800");
        assert_eq!(tid, "2437");
        assert_eq!(level, "E");
        assert_eq!(tag, "GnssHAL_GnssInterface");
        assert_eq!(message, "gnssSvStatusCb: a: input svInfo.flags is 8")
    }

    #[test]
    fn regex_log_line_3() {
        let str_log: &str = "05-19 06:49:59.836  2045  5774 I ActivityTaskManager: START u0 {act=android.intent.action.MAIN cat=[android.intent.category.HOME] flg=0x10000000 cmp=com.google.android.apps.nexuslauncher/.NexusLauncherActivity (has extras)} from uid 10092";

        let caps = LOG_LINE.captures(str_log).unwrap();
        let date = caps.name("date").map_or("", |s| s.as_str());
        let time = caps.name("time").map_or("", |s| s.as_str());
        let owner = caps.name("owner").map_or("", |s| s.as_str());
        let tid = caps.name("tid").map_or("", |s| s.as_str());
        let level = caps.name("level").map_or("", |s| s.as_str());
        let tag = caps.name("tag").map_or("", |s| s.as_str().trim());
        let message = caps.name("message").map_or("", |s| s.as_str().trim());

        assert_eq!(date, "05-19");
        assert_eq!(time, "06:49:59.836");
        assert_eq!(owner, "2045");
        assert_eq!(tid, "5774");
        assert_eq!(level, "I");
        assert_eq!(tag, "ActivityTaskManager");
        assert_eq!(message, "START u0 {act=android.intent.action.MAIN cat=[android.intent.category.HOME] flg=0x10000000 cmp=com.google.android.apps.nexuslauncher/.NexusLauncherActivity (has extras)} from uid 10092")
    }

    #[test]
    fn regex_pid_start_5_1_brief() {
        let str_log: &str = "I/ActivityManager( 2045): Start proc 10212:com.google.android.gms.ui/u0a102 for service {com.google.android.gms/com.google.android.gms.chimera.UiIntentOperationService}";

        let caps = PID_START_5_1.captures(str_log).unwrap();
        let line_pid = caps.name("line_pid").map_or("", |s| s.as_str());
        let line_package = caps.name("line_package").map_or("", |s| s.as_str().trim());
        let target = caps.name("target").map_or("", |s| s.as_str());

        assert_eq!(line_pid, "10212");
        assert_eq!(line_package, "com.google.android.gms.ui");
        assert_eq!(target, "service {com.google.android.gms/com.google.android.gms.chimera.UiIntentOperationService}")
    }

    #[test]
    fn regex_pid_start_5_1() {
        let str_log: &str = "05-18 22:25:17.632  2045  2074 I ActivityManager: Start proc 18990:com.example.test.dev/u0a136 for activity {com.example.test.dev/com.example.test.presentation.main.MainActivity}";

        let caps = PID_START_5_1.captures(str_log).unwrap();
        let line_pid = caps.name("line_pid").map_or("", |s| s.as_str());
        let line_package = caps.name("line_package").map_or("", |s| s.as_str().trim());
        let target = caps.name("target").map_or("", |s| s.as_str());

        assert_eq!(line_pid, "18990");
        assert_eq!(line_package, "com.example.test.dev");
        assert_eq!(
            target,
            "activity {com.example.test.dev/com.example.test.presentation.main.MainActivity}"
        )
    }

    #[test]
    fn regex_pid_kill() {
        let str_log: &str =
            "Killing 8822:com.google.android.apps.maps/u0a120 (adj 985): empty for 2733s";

        let caps = PID_KILL.captures(str_log).unwrap();
        let pid = caps.name("pid").map_or("", |s| s.as_str());
        let package_line = caps.name("package_line").map_or("", |s| s.as_str());

        assert_eq!(pid, "8822");
        assert_eq!(package_line, "com.google.android.apps.maps");
    }

    #[test]
    fn regex_pid_death() {
        let str_log: &str = "Process com.example.urg (pid 7404) has died";

        let caps = PID_DEATH.captures(str_log).unwrap();
        let pid = caps.name("pid").map_or("", |s| s.as_str());
        let package_line = caps.name("package_line").map_or("", |s| s.as_str());

        assert_eq!(pid, "7404");
        assert_eq!(package_line, "com.example.urg");
    }
}
