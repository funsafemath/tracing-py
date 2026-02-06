use std::{
    fs::OpenOptions,
    io::{stderr, stdout},
};

use pyo3::{Bound, PyResult};
use time::UtcOffset;
use tracing::Level;
use tracing_appender::{
    non_blocking::{NonBlockingBuilder, WorkerGuard},
    rolling::RollingFileAppender,
};
use tracing_core::LevelFilter;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{
        self, FormatEvent, FormatFields, MakeWriter,
        format::{self, DefaultFields, Format},
        time::{FormatTime, OffsetTime, Uptime, UtcTime},
    },
};

use crate::layer::{
    ThreadSafeLayer,
    fmt::{
        FmtLayer, LogFile, PyFormat,
        file::NonBlocking,
        time::{PyTimer, TimeFormat},
    },
};

pub trait ToDynLayer {
    fn dyn_layer(&self) -> PyResult<(Box<dyn ThreadSafeLayer>, Option<WorkerGuard>)>;
}

impl ToDynLayer for Bound<'_, FmtLayer> {
    fn dyn_layer(&self) -> PyResult<(Box<dyn ThreadSafeLayer>, Option<WorkerGuard>)> {
        let FmtLayer {
            log_level,
            file,
            format,
            fmt_span,
            non_blocking,
            log_internal_errors,
            timer,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
        } = &*self.borrow();

        // chaining methods would be more elegant, but it requires guessing the default values
        let mut layer: fmt::Layer<Registry> = fmt::layer();

        if let Some(log_internal_errors) = log_internal_errors {
            layer = layer.log_internal_errors(*log_internal_errors);
        }

        if let Some(with_ansi) = with_ansi {
            layer = layer.with_ansi(*with_ansi);
        }

        if let Some(with_file) = with_file {
            layer = layer.with_file(*with_file);
        }

        if let Some(with_level) = with_level {
            layer = layer.with_level(*with_level);
        }

        if let Some(with_line_number) = with_line_number {
            layer = layer.with_line_number(*with_line_number);
        }

        if let Some(with_target) = with_target {
            layer = layer.with_target(*with_target);
        }

        if let Some(with_thread_ids) = with_thread_ids {
            layer = layer.with_thread_ids(*with_thread_ids);
        }

        layer = layer.with_span_events(fmt_span.clone());

        set_writer_and_rest(
            layer,
            *log_level,
            *format,
            timer.as_ref(),
            file,
            *non_blocking,
        )
    }
}

type RFmtLayer<N, E, T, W> = fmt::Layer<Registry, N, Format<E, T>, W>;
trait Writer = for<'writer> MakeWriter<'writer> + Send + Sync + 'static;
trait TimeFmt = fmt::time::FormatTime + Send + Sync + 'static;
trait FieldFmt = Send + Sync + 'static + for<'a> FormatFields<'a>;
trait LogFmt = Send + Sync + 'static;

// please help me
fn set_level_and_finish<F, L, T, W>(
    layer: RFmtLayer<F, L, T, W>,
    level: Level,
) -> Box<dyn ThreadSafeLayer>
where
    Format<L, T>: FormatEvent<Registry, F>,
    F: FieldFmt,
    L: LogFmt,
    T: FormatTime + Send + Sync + 'static,
    W: Writer,
{
    // no need to use a filter that filters nothing
    if level == Level::TRACE {
        Box::new(layer)
    } else {
        Box::new(layer.with_filter(LevelFilter::from(level)))
    }
}

// ...please
fn set_format_and_rest<F, L, T, W>(
    layer: RFmtLayer<F, L, T, W>,
    level: Level,
    format: PyFormat,
) -> Box<dyn ThreadSafeLayer>
where
    Format<L, T>: FormatEvent<Registry, F>,
    F: FieldFmt,
    L: LogFmt,
    T: FormatTime + Send + Sync + 'static,
    W: Writer,
{
    match format {
        PyFormat::Full => set_level_and_finish(layer, level),
        PyFormat::Compact => {
            set_level_and_finish::<F, format::Compact, T, W>(layer.compact(), level)
        }
        PyFormat::Pretty => {
            set_level_and_finish::<format::Pretty, format::Pretty, T, W>(layer.pretty(), level)
        }
        PyFormat::Json => {
            set_level_and_finish::<format::JsonFields, format::Json, T, W>(layer.json(), level)
        }
    }
}

// this is literally typeslop, who thought using types to parametrize your structs is a good idea
fn set_without_time_and_rest<W>(
    layer: fmt::Layer<Registry, DefaultFields, Format<format::Full>, W>,
    level: Level,
    format: PyFormat,
    timer: Option<&PyTimer>,
) -> Box<dyn ThreadSafeLayer>
where
    W: Writer,
{
    match timer {
        Some(fmt) => match fmt.timer() {
            super::time::Timer::SystemTime => set_format_and_rest(layer, level, format),
            super::time::Timer::Uptime => {
                set_format_and_rest(layer.with_timer(Uptime::default()), level, format)
            }
            super::time::Timer::Custom(time, time_format) => match time {
                super::time::Time::Utc => match &time_format {
                    TimeFormat::Custom(owned_format_item) => set_format_and_rest(
                        layer.with_timer(UtcTime::new(owned_format_item.clone())),
                        level,
                        format,
                    ),
                    TimeFormat::Predefined(predefined) => set_format_and_rest(
                        layer.with_timer(UtcTime::new(*predefined)),
                        level,
                        format,
                    ),
                    TimeFormat::Rfc3339 => {
                        set_format_and_rest(layer.with_timer(UtcTime::rfc_3339()), level, format)
                    }
                },
                super::time::Time::Local => match time_format {
                    TimeFormat::Custom(owned_format_item) => set_format_and_rest(
                        layer.with_timer(OffsetTime::new(
                            UtcOffset::current_local_offset().unwrap(),
                            owned_format_item.clone(),
                        )),
                        level,
                        format,
                    ),
                    TimeFormat::Predefined(predefined) => set_format_and_rest(
                        layer.with_timer(OffsetTime::new(
                            UtcOffset::current_local_offset().unwrap(),
                            *predefined,
                        )),
                        level,
                        format,
                    ),
                    TimeFormat::Rfc3339 => set_format_and_rest(
                        layer.with_timer(OffsetTime::local_rfc_3339().unwrap()),
                        level,
                        format,
                    ),
                },
            },
        },
        None => set_format_and_rest(layer.without_time(), level, format),
    }
}

// okay, it may be a good idea, but it's a nightmare to configure such types at runtime
// mainly because all used types must be present during the compile time, yes
fn set_writer_and_rest(
    layer: fmt::Layer<Registry>,
    level: Level,
    format: PyFormat,
    timer: Option<&PyTimer>,
    file: &LogFile,
    nonblocking: Option<NonBlocking>,
) -> PyResult<(Box<dyn ThreadSafeLayer>, Option<WorkerGuard>)>
where
{
    let mut opts = OpenOptions::new();
    let opts = opts.create(true).append(true);

    Ok(if let Some(nonblocking) = nonblocking {
        let builder = NonBlockingBuilder::default();
        let builder = match nonblocking {
            NonBlocking::Lossy => builder.lossy(true),
            NonBlocking::Complete => builder.lossy(false),
        };

        let (writer, guard) = match file {
            LogFile::Stdout => builder.finish(stdout()),
            LogFile::Stderr => builder.finish(stderr()),
            LogFile::Path(path) => {
                let file = opts.open(path)?;
                builder.finish(file)
            }
            LogFile::Rolling(rolling) => builder.finish(RollingFileAppender::new(
                rolling.rotation.clone(),
                rolling.dir.clone(),
                rolling.prefix.clone(),
            )),
        };
        let layer = set_without_time_and_rest(layer.with_writer(writer), level, format, timer);
        (layer, Some(guard))
    } else {
        let layer = match file {
            LogFile::Stdout => {
                set_without_time_and_rest(layer.with_writer(stdout), level, format, timer)
            }
            LogFile::Stderr => {
                set_without_time_and_rest(layer.with_writer(stderr), level, format, timer)
            }
            LogFile::Path(path) => {
                let file = opts.open(path)?;
                set_without_time_and_rest(layer.with_writer(file), level, format, timer)
            }
            LogFile::Rolling(rolling) => {
                let rolling = RollingFileAppender::new(
                    rolling.rotation.clone(),
                    rolling.dir.clone(),
                    rolling.prefix.clone(),
                );
                set_without_time_and_rest(layer.with_writer(rolling), level, format, timer)
            }
        };
        (layer, None)
    })
}
// todo: rewrite functions above using macros
