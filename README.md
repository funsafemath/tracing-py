Project goals:

- obvious api

- performance

- minimal overhead if the event is not logged due to its level

- t-strings support (actually it's why I started writing this library)

- non-blocking logging, so the calling thread spends time only on the str()
  calls (if the event level >= minimal, of course), and all I/O happens on a
  separate thread (not yet done,
  https://docs.rs/tracing-appender/latest/tracing_appender/ will be used)

...

# Performance

Quite fast, despite some awful things this library does. Rust compiler
developers and tracing developers made an incredible job

# Template strings

## Notes

It was originally intended to pass template string expressions as separate event
fields, so something like t"{hello}" would be emitted as message="hello",
hello=(string representation of hello) or as message="{hello}", hello=(string
representation of hello), but in my opinion it'll often clutter logs with messy
field names (especially if using nested expressions, e.g. t"{hello + name}"),
and it's better if the developers specify the names themselves.

If it's implemented, it may be a good idea to use format specifiers to
opt-in/opt-out of this feature, so t"{hello!a}" gets included as
hello=str(hello), while t"{hello}" is not (or vice versa, not sure which is
better)

## Performance

Although printf-style formatting (passing "%s", (arg,)) is inconvenient, it is,
sadly, faster than template-strings, even if the messages are not printed: the
construction of the template object takes more time than the construction of a
tuple (let's hope CPython developres will optimize in the future).

So either use the percent formatting, or use a constant as the main message and
pass any additional information as kwargs (not a bad idea!) if you care about
these bits of performance, though if you use Python, you probably do not.

# Logging notes

For now, integers are logged as strings, because the `valuable` crate doesn't
have a big integer type. It's possible to log integers that are less than 128
bits as integers, but it's a bit inconsistent. Maybe I'll fix this later

# Warning

this library leaks memory. unbounded amount if you misuse it.

so the current implementation leaks an object for each unique callsite there is.
A callsite is identified by the bytecode instruction address, logging level, a
set of logged fields, and the kind of the callsite (event/span, spans are
currenly available only through the instrument decorator). Also filenames,
keyword parameter names are leaked (one time for each unique string, of course),
and field combinations are leaked, too.

This means you should not:

- Use tracing in dynamically compiled code (eval)

- Pass **kwargs to logging functions if you except the number of possible kwarg
  names or their different combinations to be very large (calling with **{a:
  ..., b: ...}, **{a: ..., c: ...}, **{b: ..., c: ...} will cause arrays [&a,
  &b], [&a, &c], [&b, &c] to be leaked; also at this moment different
  permutations are leaked, too, so creating kwargs from unordered collections
  may cause problems; by the way, according to the language specification, both
  passed kwargs and dicts are ordered)

- Create tons of instrumented functions with different combinations of skipped
  parameters, as arrays with references to parameter names are leaked, too. It's
  okay, though, to instrument lambdas/function objects with a fixed number of
  skipped parameters, they won't create new callsites, as they share the same
  bytecode

If you are not doing any of these, you should be fine. If you leak too much
objects (>=100000), you'll see a warning each time a new object is leaked.

# @instrument() notes

- when instrumenting async functions/generators, a span is created on the
  original function call, even before the first poll/next call

- also if an instrumented function return value is a generator/coroutine, it's
  always instrumented. This behaviour may be unexpected, but I've yet to find a
  good way to reliably determine if a function is a generator or an async
  function, and not a normal function that returns a generator/coroutine.
  Manually passing async/generator=True is too much boilerplate and can be
  easily forgotten; inspect's isgenerator/iscoroutine are broken by most
  used-defined decorators, including functools.wraps, so it's not an option. In
  fact, I'm not sure it's even possible, as a decorated async function is pretty
  much a normal function that returns a coroutine, so how am I expected to
  differentiate between a function that returns a coroutine and a function that
  returns a coroutine?

# Some missing features I want to add

Colorful error logging with context,
[color-eyre](https://crates.io/crates/color-eyre)-like
