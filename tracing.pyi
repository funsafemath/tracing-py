# Docstrings are copied from the tracing crate

from string.templatelib import Template
from typing import Any, Callable, Self, Sequence, TypeVar, overload

# todo: pick either uppercase or camelcase for enum-like classes
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

    Full: Format
    """The default formatter.
    
    This emits human-readable,
    single-line logs for each event that occurs, with the current span context
    displayed before the formatted representation of the event.
    """

    Compact: Format
    """A variant of the default formatter, optimized for short line lengths.
    
    Fields from the current span context are appended to
    the fields of the formatted event, and span names are not shown; the
    verbosity level is abbreviated to a single character.
    """

    Pretty: Format
    """Emits excessively pretty, multi-line logs, optimized for human readability.

    This is primarily intended to be used in local
    development and debugging, or for command-line applications, where
    automated analysis and compact storage of logs is less of a priority than
    readability and visual appeal. See [here](Pretty#example-output)
    for sample output.
    """

    Json: Format
    """
    Outputs newline-delimited JSON logs.

    This is intended
    for production use with systems where structured logs are consumed as JSON
    by analysis and viewing tools. The JSON output is not optimized for human
    readability. See [here](Json#example-output) for sample output.
    """

class FmtLayer:
    def __new__(
        cls,
        *,
        log_internal_errors: bool | None = None,
        with_ansi: bool | None = None,
        with_file: bool | None = None,
        with_level: bool | None = None,
        with_line_number: bool | None = None,
        with_target: bool | None = None,
        with_thread_ids: bool | None = None,
        with_max_level: bool | None = None,
        without_time: bool = False,
        fmt_span: FmtSpan = FmtSpan.NONE,
        format: Format = Format.Full,
    ) -> Self: ...

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
    level: Level | None = None,
    skip: Sequence[str] | None = None,
    skip_all: bool | None = None,
) -> Callable[[T], T]: ...
@overload
def instrument(func: T) -> T: ...
