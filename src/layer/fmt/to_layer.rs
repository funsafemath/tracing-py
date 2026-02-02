use std::{
    fs::OpenOptions,
    io::{stderr, stdout},
};

use pyo3::{Bound, PyResult};
use tracing::Level;
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_core::LevelFilter;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{
        self, FormatEvent, FormatFields, MakeWriter,
        format::{self},
    },
};

use crate::layer::{
    ThreadSafeLayer,
    fmt::{FmtLayer, Format, LogFile, NonBlocking},
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
            without_time,
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
            *without_time,
            file,
            *non_blocking,
        )
    }
}

type RFmtLayer<N, E, T, W> = fmt::Layer<Registry, N, format::Format<E, T>, W>;
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
    format::Format<L, T>: FormatEvent<Registry, F>,
    F: FieldFmt,
    L: LogFmt,
    T: TimeFmt,
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
    format: Format,
) -> Box<dyn ThreadSafeLayer>
where
    format::Format<L, T>: FormatEvent<Registry, F>,
    F: FieldFmt,
    L: LogFmt,
    T: TimeFmt,
    W: Writer,
{
    match format {
        Format::Full => set_level_and_finish(layer, level),
        Format::Compact => set_level_and_finish::<F, format::Compact, T, W>(layer.compact(), level),
        Format::Pretty => {
            set_level_and_finish::<format::Pretty, format::Pretty, T, W>(layer.pretty(), level)
        }
        Format::Json => {
            set_level_and_finish::<format::JsonFields, format::Json, T, W>(layer.json(), level)
        }
    }
}

// this is literally typeslop, who thought using types to parametrize your structs is a good idea
fn set_without_time_and_rest<F, L, T, W>(
    layer: RFmtLayer<F, L, T, W>,
    level: Level,
    format: Format,
    without_time: bool,
) -> Box<dyn ThreadSafeLayer>
where
    format::Format<L, T>: FormatEvent<Registry, F>,
    format::Format<L, ()>: FormatEvent<Registry, F>,
    F: FieldFmt,
    L: LogFmt,
    T: TimeFmt,
    W: Writer,
{
    if without_time {
        set_format_and_rest(layer.without_time(), level, format)
    } else {
        set_format_and_rest(layer, level, format)
    }
}

// okay, it may be a good idea, but it's a nightmare to configure such types at runtime
// mainly because all used types must be present during the compile time, yes
fn set_writer_and_rest<F, L, T, W>(
    layer: RFmtLayer<F, L, T, W>,
    level: Level,
    format: Format,
    without_time: bool,
    file: &LogFile,
    nonblocking: Option<NonBlocking>,
) -> PyResult<(Box<dyn ThreadSafeLayer>, Option<WorkerGuard>)>
where
    format::Format<L, T>: FormatEvent<Registry, F>,
    format::Format<L, ()>: FormatEvent<Registry, F>,
    F: FieldFmt,
    L: LogFmt,
    T: TimeFmt,
    W: Writer,
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
            LogFile::Stdout => {
                let (writer, guard) = builder.finish(stdout());
                (
                    set_without_time_and_rest(
                        layer.with_writer(writer),
                        level,
                        format,
                        without_time,
                    ),
                    guard,
                )
            }
            LogFile::Stderr => {
                let (writer, guard) = builder.finish(stderr());
                (
                    set_without_time_and_rest(
                        layer.with_writer(writer),
                        level,
                        format,
                        without_time,
                    ),
                    guard,
                )
            }
            LogFile::Path(path) => {
                let file = opts.open(path)?;
                let (writer, guard) = builder.finish(file);
                (
                    set_without_time_and_rest(
                        layer.with_writer(writer),
                        level,
                        format,
                        without_time,
                    ),
                    guard,
                )
            }
        };
        (writer, Some(guard))
    } else {
        let writer = match file {
            LogFile::Stdout => {
                set_without_time_and_rest(layer.with_writer(stdout), level, format, without_time)
            }
            LogFile::Stderr => {
                set_without_time_and_rest(layer.with_writer(stderr), level, format, without_time)
            }
            LogFile::Path(path) => {
                let file = opts.open(path)?;
                set_without_time_and_rest(layer.with_writer(file), level, format, without_time)
            }
        };
        (writer, None)
    })
}
// todo: rewrite functions above using macros
