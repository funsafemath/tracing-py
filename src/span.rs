//! Spans represent periods of time in which a program was executing in a
//! particular context.
//!
//! See [tracing::span] for more details
//!
//! # Implementation notes
//!
//! [Subscribers](tracing::Subscriber) may need to know the spans parent,
//! typically the span parent is the last opened span from a current thread,
//! which is correct, under the assumption that the function can't return mid-execution and then continue executing.
//!
//! The problem is that this is exactly what asynchronous code breaks (for the good):
//! in case of state machine-based async an async function is can roughly be represented as several functions
//! obtained by splitting the original one at the await points.
//!
//! So if there are two unrelated async functions A and B, the following is possible:
//! function A opens a span, yields, function B is scheduled and opens a span, which parent is incorrectly set to the span
//! opened by the function A.
//! Tracing solves this problem with its [instrument function](tracing::Instrument::instrument), which wraps a future
//! into one that enters a span each time it's polled
//!
//! There's a good and simple approach which can be used with asyncio:
//! https://docs.python.org/3/c-api/contextvars.html, we can store the last span in a context varible,
//! it's not truly safe, as yields do not care about contextvars, as may some other async runtimes
//!
//! For now, the only way to create a span from Python is to use the instrument function,
//! though if it turns out to be unusable (I don't think @instrument is much less usable than with span: ..., which
//! adds an additional indentation level to the all spanned code),
//! or if I find a performant way to address the forementioned issue, I'll add others

use pyo3::prelude::*;
use tracing::info;
#[pyclass]
struct Span(Option<tracing::Span>);

#[pymethods]
impl Span {
    fn __enter__(&mut self) -> EnteredSpan {
        dbg!("enter");
        EnteredSpan(Some(self.0.take().unwrap()))
    }
}

#[pyclass]
#[allow(unused)]
struct EnteredSpan(Option<tracing::Span>);

impl EnteredSpan {
    fn __exit__(&mut self) {
        dbg!("exit");
        drop(self.0.take().unwrap());
    }
}

#[pyfunction]
fn span_error() {}
