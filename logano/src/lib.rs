use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use chrono_tz::Tz;
use log::Level;
use once_cell::sync::Lazy;

static GLOBAL_TIMEZONE: Lazy<RwLock<Tz>> = Lazy::new(|| { RwLock::new(Tz::UTC) });

pub fn set_global_timezone(tz: Tz) {
    let mut write_lock = GLOBAL_TIMEZONE.write().unwrap();
    *write_lock = tz;
}

pub fn get_global_timezone() -> Tz {
    let read_lock = GLOBAL_TIMEZONE.read().unwrap();
    *read_lock
}

/// Log Data Point
pub enum Ldp {
    NamedString(String, String),
    NamedU64(String, u64),
}

impl From<(&str, &str)> for Ldp {
    fn from(pair: (&str, &str)) -> Self {
        let (name, value) = pair;
        Ldp::NamedString(String::from(name), String::from(value))
    }
}

impl From<(&str, String)> for Ldp {
    fn from(pair: (&str, String)) -> Self {
        let (name, value) = pair;
        Ldp::NamedString(String::from(name), value)
    }
}

impl From<(&str, u64)> for Ldp {
    fn from(pair: (&str, u64)) -> Self {
        let (name, value) = pair;
        Ldp::NamedU64(String::from(name), value)
    }
}

fn default_formatter(timestamp_ns: u64, level: Level, message: Option<String>, ldps: Vec<Ldp>) -> String {
    let mut obj = json::JsonValue::new_object();
    obj["time_ns"] = timestamp_ns.into();
    obj["time"] =
        chrono::TimeZone::timestamp_nanos(&get_global_timezone(), timestamp_ns as i64)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            .into();
    obj["level"] = level.to_string().into();
    if let Some(text) = message {
        obj["message"] = text.into();
    }
    for ldp in ldps {
        match ldp {
            Ldp::NamedString(name, value) => obj[name] = value.into(),
            Ldp::NamedU64(name, value) => obj[name] = value.into(),
        }
    }
    obj.dump()
}

#[cfg(test)]
#[test]
fn test_default_formatter() {
    // Generate data with:
    // $ date -u '+%s %Y-%m-%dT%H:%M:%S%z'
    // 1591847174 2020-06-11T03:46:14+0000
    // $ date '+%s %Y-%m-%dT%H:%M:%S%z'
    // 1591847197 2020-06-10T20:46:37-0700

    // timestamp_ns
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO"}"#,
               default_formatter(0, Level::Info, None, Vec::new()));
    assert_eq!(r#"{"time_ns":1,"time":"1970-01-01T00:00:00.000Z","level":"INFO"}"#,
               default_formatter(1, Level::Info, None, Vec::new()));
    assert_eq!(r#"{"time_ns":1000000,"time":"1970-01-01T00:00:00.001Z","level":"INFO"}"#,
               default_formatter(1_000_000, Level::Info, None, Vec::new()));
    assert_eq!(r#"{"time_ns":1591847174000000000,"time":"2020-06-11T03:46:14.000Z","level":"INFO"}"#,
               default_formatter(1591847174_000_000_000, Level::Info, None, Vec::new()));

    // level
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"DEBUG"}"#,
               default_formatter(0, Level::Debug, None, Vec::new()));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"ERROR"}"#,
               default_formatter(0, Level::Error, None, Vec::new()));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO"}"#,
               default_formatter(0, Level::Info, None, Vec::new()));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"TRACE"}"#,
               default_formatter(0, Level::Trace, None, Vec::new()));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"WARN"}"#,
               default_formatter(0, Level::Warn, None, Vec::new()));

    // Message
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","message":""}"#,
               default_formatter(0, Level::Info, Some(String::new()), Vec::new()));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","message":"message1"}"#,
               default_formatter(0, Level::Info, Some(String::from("message1")), Vec::new()));

    // NamedString
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","name1":"value1"}"#,
               default_formatter(0, Level::Info, None, vec!(("name1", "value1").into())));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","name1":"value1"}"#,
               default_formatter(0, Level::Info, None, vec!(("name1", String::from("value1")).into())));

    // NamedU64
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","name1":0}"#,
               default_formatter(0, Level::Info, None, vec!(("name1", 0).into())));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","name1":123}"#,
               default_formatter(0, Level::Info, None, vec!(("name1", 123).into())));
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","name1":18446744073709551615}"#,
               default_formatter(0, Level::Info, None, vec!(("name1", 0xFFFF_FFFF_FFFF_FFFF).into())));

    // Test order
    assert_eq!(r#"{"time_ns":0,"time":"1970-01-01T00:00:00.000Z","level":"INFO","message":"message1","keyB":456,"keyA":123}"#,
               default_formatter(0, Level::Info, Some("message1".into()), vec!(("keyB", 456).into(), ("keyA", 123).into())));
}


pub type GlobalFormatterFn = fn(u64, Level, Option<String>, Vec<Ldp>) -> String;

static GLOBAL_FORMATTER: Lazy<ArcSwap<GlobalFormatterFn>> =
    Lazy::new(|| { ArcSwap::new(Arc::new(default_formatter)) });

pub fn set_global_formatter(f: GlobalFormatterFn) {
    GLOBAL_FORMATTER.store(Arc::new(f));
}

// trait LogWriter {
//     fn write(formatted_line: String);
// }
//
// static GLOBAL_WRITER: ArcSwap<GlobalFormatter> = ArcSwap::new(Arc::new(GlobalFormatter::Default));
//
// pub fn set_global_formatter(f: GlobalFormatterFn) {
//     GLOBAL_FORMATTER.store(Arc::new(GlobalFormatter::Fn(f)));
// }
//
