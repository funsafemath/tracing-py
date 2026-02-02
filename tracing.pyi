# Most docstrings are copied from the tracing crate

from string.templatelib import Template
from typing import Any, Callable, Self, Sequence, TypeVar, overload

class Level:
    """Describes the level of verbosity of a span or event."""

    ERROR: Level
    """The "error" level.
    
    Designates very serious errors.
    """

    WARN: Level
    """The "warn" level.
    
    Designates hazardous situations.
    """

    INFO: Level
    """The "info" level.
    
    Designates useful information.
    """

    DEBUG: Level
    """The "debug" level.
    
    Designates lower priority information.
    """

    TRACE: Level
    """The "trace" level.
    
    Designates very low priority, often extremely verbose, information.
    """

class FmtSpan:
    """Configures what points in the span lifecycle are logged as events"""

    NEW: FmtSpan
    """one event when span is created"""

    ENTER: FmtSpan
    """one event per enter of a span"""

    EXIT: FmtSpan
    """one event per exit of a span"""

    CLOSE: FmtSpan
    """one event when the span is dropped"""

    NONE: FmtSpan
    """spans are ignored (this is the default)"""

    ACTIVE: FmtSpan
    """one event per enter/exit of a span"""

    FULL: FmtSpan
    """events at all points (new, enter, exit, drop)"""

    def __or__(self, other: Self) -> Self: ...
    def __and__(self, other: Self) -> Self: ...

class Format:
    """Formatters for logging tracing events."""

    FULL: Format
    """The default formatter.
    
    This emits human-readable,
    single-line logs for each event that occurs, with the current span context
    displayed before the formatted representation of the event.
    """

    COMPACT: Format
    """A variant of the default formatter, optimized for short line lengths.
    
    Fields from the current span context are appended to
    the fields of the formatted event, and span names are not shown; the
    verbosity level is abbreviated to a single character.
    """

    PRETTY: Format
    """Emits excessively pretty, multi-line logs, optimized for human readability.

    This is primarily intended to be used in local
    development and debugging, or for command-line applications, where
    automated analysis and compact storage of logs is less of a priority than
    readability and visual appeal. See [here](Pretty#example-output)
    for sample output.
    """

    JSON: Format
    """
    Outputs newline-delimited JSON logs.

    This is intended
    for production use with systems where structured logs are consumed as JSON
    by analysis and viewing tools. The JSON output is not optimized for human
    readability. See [here](Json#example-output) for sample output.
    """

class File:
    STDOUT: File
    STDERR: File

class NonBlocking:
    """Sets whether logger should be lossy or not.

    This has effect only if the logger has reached its max capacity, which is 128_000 lines by default"""

    LOSSY: NonBlocking
    """logs will be dropped when the buffered limit is reached"""

    COMPLETE: NonBlocking
    """backpressure will be exerted on senders, blocking them until the buffer has capacity again"""

class FmtLayer:
    def __new__(
        cls,
        *,
        log_level: Level = Level.INFO,
        file: File | str = File.STDOUT,
        format: Format = Format.FULL,
        fmt_span: FmtSpan = FmtSpan.NONE,
        non_blocking: NonBlocking | None = None,
        log_internal_errors: bool | None = None,
        without_time: bool = False,
        with_ansi: bool | None = None,
        with_file: bool | None = None,
        with_level: bool | None = None,
        with_line_number: bool | None = None,
        with_target: bool | None = None,
        with_thread_ids: bool | None = None,
    ) -> Self: ...
    """
    Creates a new FmtLayer

        log_level
            creates a filter that passes only the events with level <= log_level,
            ERROR < WARN < INFO < DEBUG < TRACE

        non_blocking
            if none, tracing will log messages in blocking mode, 
            if passed, I/O will be delegated to a separate non-GIL-bound thread
    """

def init(registry: FmtLayer | Sequence[FmtLayer] | None = None) -> None: ...
@overload
def trace(message: Template | str | Any | None = None, **kwargs) -> None: ...
@overload
def trace(message: str, fmt_args: tuple[Any, ...], **kwargs) -> None: ...
@overload
def debug(message: Template | str | Any | None = None, **kwargs) -> None: ...
@overload
def debug(message: str, fmt_args: tuple[Any, ...], **kwargs) -> None: ...
@overload
def info(message: Template | str | Any | None = None, **kwargs) -> None: ...
@overload
def info(message: str, fmt_args: tuple[Any, ...], **kwargs) -> None: ...
@overload
def warn(message: Template | str | Any | None = None, **kwargs) -> None: ...
@overload
def warn(message: str, fmt_args: tuple[Any, ...], **kwargs) -> None: ...
@overload
def error(message: Template | str | Any | None = None, **kwargs) -> None: ...
@overload
def error(message: str, fmt_args: tuple[Any, ...], **kwargs) -> None: ...

T = TypeVar("T", bound=Callable)

@overload
def instrument(
    func: None = None,
    /,
    *,
    level: Level = Level.INFO,
    skip: Sequence[str] = [],
    skip_all: bool = False,
    ret: bool = False,
    ret_err_only: bool = False,
    no_yield: bool = False,
) -> Callable[[T], T]: ...
@overload
def instrument(
    func: T,
    /,
    *,
    level: Level = Level.INFO,
    skip: Sequence[str] = [],
    skip_all: bool = False,
    ret: bool = False,
    ret_err_only: bool = False,
    no_yield: bool = False,
) -> T: ...

__all__ = [
    "init",
    "instrument",
    "trace",
    "debug",
    "info",
    "warn",
    "error",
    "Level",
    "FmtLayer",
    "Format",
    "File",
    "FmtSpan",
    "NonBlocking",
]
