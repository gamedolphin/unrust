use std::{ffi::c_char, io::Write};

use parking_lot::Once;
use tracing::{Level, Metadata};
use tracing_subscriber::{
    fmt::{
        self,
        format::{Compact, DefaultFields, Format},
        Layer, MakeWriter,
    },
    prelude::*,
    reload, Registry,
};

type Reloader =
    reload::Handle<Layer<Registry, DefaultFields, Format<Compact, ()>, MakeLogger>, Registry>;

static mut RELOAD: Option<Reloader> = None;
static INIT: Once = Once::new();

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Error = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
}

pub type LoggerFunc = extern "C" fn(level: LogLevel, str: *mut c_char, len: usize);

type LogFn = dyn Fn(LogLevel, String) + Send + Sync + 'static;

type LogCallback = Option<Box<LogFn>>;

fn get_cached_reloader() -> &'static Reloader {
    unsafe {
        INIT.call_once(|| {
            let layer = fmt::layer()
                .without_time()
                .with_ansi(false)
                .compact()
                .with_writer(MakeLogger { logger: None });
            let (layer, reload_handle) = reload::Layer::new(layer);
            tracing_subscriber::registry().with(layer).init();

            RELOAD = Some(reload_handle);
        });

        if let Some(val) = &RELOAD {
            val
        } else {
            panic!("reload not set")
        }
    }
}

pub(crate) fn setup_logging(logger: Box<LogFn>) {
    let reloader = get_cached_reloader();
    let _ = reloader.modify(|layer| {
        *layer.writer_mut() = MakeLogger {
            logger: Some(logger),
        };
    });
}

pub(crate) fn teardown_logging() {
    let reloader = get_cached_reloader();
    let _ = reloader.modify(|layer| {
        *layer.writer_mut() = MakeLogger {
            logger: LogCallback::default(),
        };
    });
}

struct MakeLogger {
    logger: LogCallback,
}

enum UnityLogger<'a> {
    Error(&'a LogCallback),
    Warning(&'a LogCallback),
    Info(&'a LogCallback),
    Debug(&'a LogCallback),
}

impl<'a> Write for UnityLogger<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let str = String::from_utf8_lossy(buf).to_string();

        let (level, log) = match self {
            UnityLogger::Error(log) => (LogLevel::Error, log),
            UnityLogger::Warning(log) => (LogLevel::Warning, log),
            UnityLogger::Info(log) => (LogLevel::Info, log),
            UnityLogger::Debug(log) => (LogLevel::Debug, log),
        };

        if let Some(logger) = log {
            (logger)(level, str)
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for MakeLogger {
    type Writer = UnityLogger<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        // We must have an implementation of `make_writer` that makes
        // a "default" writer without any configuring metadata. Let's
        // just return stdout in that case.
        UnityLogger::Info(&self.logger)
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        match *meta.level() {
            Level::DEBUG => UnityLogger::Debug(&self.logger),
            Level::ERROR => UnityLogger::Error(&self.logger),
            Level::INFO => UnityLogger::Info(&self.logger),
            Level::WARN => UnityLogger::Warning(&self.logger),
            Level::TRACE => UnityLogger::Info(&self.logger),
        }
    }
}
