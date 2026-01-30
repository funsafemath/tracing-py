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
    FmtLayer, Format,
    fmt::{LogFile, NonBlocking},
};

pub(crate) trait DynLayer {
    fn dyn_layer(&self) -> PyResult<(Box<dyn Layer<Registry> + Send + Sync>, Option<WorkerGuard>)>;
}

impl<'py> DynLayer for Bound<'py, FmtLayer> {
    fn dyn_layer(&self) -> PyResult<(Box<dyn Layer<Registry> + Send + Sync>, Option<WorkerGuard>)> {
        let FmtLayer {
            log_internal_errors,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
            with_max_level,
            without_time,
            fmt_span,
            format,
            file,
            non_blocking,
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
            Level::from(*with_max_level),
            *format,
            *without_time,
            file,
            *non_blocking,
        )
    }
}

type RFmtLayer<N, E, T, W> = fmt::Layer<Registry, N, format::Format<E, T>, W>;

// please help me
fn set_level_and_finish<N, E, T, W>(
    layer: RFmtLayer<N, E, T, W>,
    level: Level,
) -> Box<dyn Layer<Registry> + Send + Sync>
where
    format::Format<E, T>: FormatEvent<Registry, N>,
    N: Send + Sync + 'static + for<'a> FormatFields<'a>,
    E: Send + Sync + 'static,
    T: Send + Sync + 'static,
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    // no need to use a filter that filters nothing
    if level == Level::TRACE {
        Box::new(layer)
    } else {
        Box::new(layer.with_filter(LevelFilter::from(level)))
    }
}

// ...please
fn set_format_and_rest<N, E, T, W>(
    layer: RFmtLayer<N, E, T, W>,
    level: Level,
    format: Format,
) -> Box<dyn Layer<Registry> + Send + Sync>
where
    format::Format<E, T>: FormatEvent<Registry, N>,
    N: Send + Sync + 'static + for<'a> FormatFields<'a>,
    E: Send + Sync + 'static,
    T: Send + Sync + fmt::time::FormatTime + 'static,
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    match format {
        Format::Full => set_level_and_finish::<N, E, T, W>(layer, level),
        Format::Compact => set_level_and_finish::<N, format::Compact, T, W>(layer.compact(), level),
        Format::Pretty => {
            set_level_and_finish::<format::Pretty, format::Pretty, T, W>(layer.pretty(), level)
        }
        Format::Json => {
            set_level_and_finish::<format::JsonFields, format::Json, T, W>(layer.json(), level)
        }
    }
}

// this is literally typeslop, who thought using types to parametrize your structs is a good idea
fn set_without_time_and_rest<N, E, T, W>(
    layer: RFmtLayer<N, E, T, W>,
    level: Level,
    format: Format,
    without_time: bool,
) -> Box<dyn Layer<Registry> + Send + Sync>
where
    format::Format<E, T>: FormatEvent<Registry, N>,
    format::Format<E, ()>: FormatEvent<Registry, N>,
    N: Send + Sync + 'static + for<'a> FormatFields<'a>,
    E: Send + Sync + 'static,
    T: Send + Sync + fmt::time::FormatTime + 'static,
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    if without_time {
        set_format_and_rest(layer.without_time(), level, format)
    } else {
        set_format_and_rest(layer, level, format)
    }
}

// okay, it may be a good idea, but it's a nightmare to configure such types at runtime
// mainly because all used types must be present during the compile time, yes
fn set_writer_and_rest<N, E, T, W>(
    layer: RFmtLayer<N, E, T, W>,
    level: Level,
    format: Format,
    without_time: bool,
    file: &LogFile,
    nonblocking: Option<NonBlocking>,
) -> PyResult<(Box<dyn Layer<Registry> + Send + Sync>, Option<WorkerGuard>)>
where
    format::Format<E, T>: FormatEvent<Registry, N>,
    format::Format<E, ()>: FormatEvent<Registry, N>,
    N: Send + Sync + 'static + for<'a> FormatFields<'a>,
    E: Send + Sync + 'static,
    T: Send + Sync + fmt::time::FormatTime + 'static,
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let mut opts = OpenOptions::new();
    let opts = opts.create(true).append(true);

    Ok(match nonblocking {
        Some(nonblocking) => {
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
                    let layer = layer.with_writer(writer);
                    (
                        set_without_time_and_rest(layer, level, format, without_time),
                        guard,
                    )
                }
            };
            (writer, Some(guard))
        }
        None => {
            let writer = match file {
                LogFile::Stdout => set_without_time_and_rest(
                    layer.with_writer(stdout),
                    level,
                    format,
                    without_time,
                ),
                LogFile::Stderr => set_without_time_and_rest(
                    layer.with_writer(stderr),
                    level,
                    format,
                    without_time,
                ),
                LogFile::Path(path) => {
                    let file = opts.open(path)?;
                    set_without_time_and_rest(layer.with_writer(file), level, format, without_time)
                }
            };
            (writer, None)
        }
    })
}
// todo: rewrite the above functions using macros
