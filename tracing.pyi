from string.templatelib import Template
from typing import Any, Callable, Self, Sequence, TypeVar, ParamSpec

# todo: pick either uppercase or camelcase for enum-like classes
class Level:
    TRACE: Level
    DEBUG: Level
    INFO: Level
    WARN: Level
    ERROR: Level

class Format:
    Full: Format
    Compact: Format
    Pretty: Format
    Json: Format

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
        format: Format = Format.Full,
    ) -> Self: ...

def init(registry: FmtLayer | Sequence[FmtLayer] | None = None) -> None: ...
def trace(message: Template | str | Any | None = None, **kwargs) -> None: ...
def debug(message: Template | str | Any | None = None, **kwargs) -> None: ...
def info(message: Template | str | Any | None = None, **kwargs) -> None: ...
def warn(message: Template | str | Any | None = None, **kwargs) -> None: ...
def error(message: Template | str | Any | None = None, **kwargs) -> None: ...

T = TypeVar("T", bound=Callable)

def instrument(func: T) -> T: ...
