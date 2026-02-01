use eyre::{ContextCompat, bail, eyre};
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyFunction, PyList, PyString, PyTuple},
};
use rapidhash::RapidHashMap;
use smallvec::smallvec;

use crate::{
    imports::get_inspect_signature,
    instrument::{
        parameter::{ParamKind, PyParameter, PyParameterMethods},
        py_signature::PySignatureMethods,
    },
    leak::{Leaker, VecLeaker},
};

// todo: it is broken if the kwarg names are invalid unicode;
// code should always return the error on invalid unicode instead of silently doing potentially incorrect things
// afaik json subscriber will emit a field twice, fmt will subscriber print it twice
// also looks like to_str -> Result<...> method doesn't solve this, it returns a lossy str even on invalid unicode
#[derive(Debug)]
pub(crate) struct Signature {
    param_names: &'static [&'static str],
    kwarg_to_index: RapidHashMap<&'static str, usize>,
    has_excess_args: bool,
    has_excess_kwargs: bool,
    pos_only: usize,
    pos_or_kw: usize,
    kw_only: usize,
}

impl Signature {
    pub(crate) fn param_names(&self) -> &'static [&'static str] {
        self.param_names
    }

    fn has_excess_args(&self) -> bool {
        self.has_excess_args
    }

    fn has_excess_kwargs(&self) -> bool {
        self.has_excess_kwargs
    }

    // todo: binding can be done more efficiently in a single pass over 0..pos_only, 0..pos_or_kw and 0..kw_only
    // though it's already >40x times faster than inspect.Signature.bind
    // todo: don't use inspect signature for default arg counts
    pub(crate) fn bind<'py>(
        &self,
        py: Python<'py>,
        args: &'py Bound<'py, PyTuple>,
        kwargs: Option<&Bound<'py, PyDict>>,
        defaults: Option<Bound<'py, PyTuple>>,
        kw_defaults: Option<Bound<'py, PyDict>>,
    ) -> eyre::Result<Vec<Bound<'py, PyAny>>> {
        // todo: move assert to the constructor
        assert!(
            self.param_names.len()
                == self.pos_only
                    + self.pos_or_kw
                    + self.kw_only
                    + self.has_excess_args as usize
                    + self.has_excess_kwargs as usize
        );

        let mut bound_args: Vec<Option<Bound<'py, PyAny>>> = vec![None; self.param_names.len()];

        let mut excess_kwargs: Vec<(Bound<'py, PyString>, Bound<'py, PyAny>)> = Vec::new();

        let mut args = args.into_iter();

        for (i, arg) in (&mut args).take(self.pos_only + self.pos_or_kw).enumerate() {
            bound_args[i] = Some(arg);
        }

        if !args.is_empty() && !self.has_excess_args() {
            bail!("too many positional arguments")
        }

        let excess_args: Vec<Bound<'py, PyAny>> = args.collect();

        if let Some(kwargs) = kwargs {
            for (kw_name, kw_value) in kwargs.iter() {
                let kw_name: Bound<'py, PyString> = kw_name.cast_into().unwrap();
                let index = self.kwarg_to_index.get(kw_name.to_str().unwrap());
                match index {
                    Some(index) => {
                        if bound_args[*index].is_some() {
                            bail!("arg \"{kw_name}\" passed twice")
                        }
                        bound_args[*index] = Some(kw_value);
                    }
                    None => {
                        if self.has_excess_kwargs() {
                            excess_kwargs.push((kw_name, kw_value));
                        } else {
                            bail!("too many keyword arguments")
                        }
                    }
                }
            }
        }

        if self.has_excess_args() {
            let list = PyList::new(py, excess_args)?;
            bound_args[self.pos_only + self.pos_or_kw] = Some(list.into_any());
        }

        if self.has_excess_kwargs() {
            let list = PyList::new(py, excess_kwargs)?;
            let last = bound_args.len() - 1;
            bound_args[last] = Some(PyDict::from_sequence(&list)?.into_any());
        }

        if let Some(defaults) = defaults {
            let defaults = defaults.into_iter().rev();
            let pos_args = &mut bound_args[..self.pos_only + self.pos_or_kw];

            for (arg, default) in pos_args.iter_mut().rev().zip(defaults) {
                if arg.is_none() {
                    *arg = Some(default);
                }
            }
        }

        if let Some(kw_defaults) = kw_defaults {
            let kw_defaults = kw_defaults.into_iter();

            for (kw_name, kw_value) in kw_defaults {
                let kw_name = kw_name
                    .cast::<PyString>()
                    .map_err(|x| eyre!("unreachable: kwarg name is not a string, {x}"))?
                    .to_str()?;
                if let Some(kw_idx) = self.kwarg_to_index.get(kw_name) {
                    let entry = &mut bound_args[*kw_idx];
                    if entry.is_none() {
                        *entry = Some(kw_value);
                    }
                }
            }
        }

        bound_args
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .wrap_err("not enough arguments")
    }
}

// inspect.signature is actually not that slow for non-native functions, and we call it only once per instrumented function
pub(crate) fn extract_signature<'py>(func: &Bound<'py, PyFunction>) -> PyResult<Signature> {
    let signature = get_inspect_signature(func.py())
        .call1((func,))?
        .cast_into()
        .map_err(|x| PyTypeError::new_err(x.to_string()))?;
    let params = signature.parameters();

    let mut param_names = smallvec![];
    let mut leaker = Leaker::acquire();
    let mut pos_only = 0;
    let mut pos_or_kw = 0;
    let mut kw_only = 0;
    let mut has_excess_args = false;
    let mut has_excess_kwargs = false;
    let mut kwarg_to_index = RapidHashMap::default();
    let mut defaults = Vec::new();

    for (i, param_tuple) in params.items()?.into_iter().enumerate() {
        let param_tuple = param_tuple.cast::<PyTuple>()?;
        let param_name = param_tuple.get_item(0)?;
        let param_name = leaker.leak_or_get(param_name.str()?.to_str()?.to_owned());
        let param = param_tuple.get_item(1)?.cast_into::<PyParameter>()?;
        match param.kind() {
            ParamKind::PositionalOnly => pos_only += 1,
            ParamKind::PositionalOrKeyword => {
                pos_or_kw += 1;
                kwarg_to_index.insert(param_name, i);
            }
            ParamKind::ExcessArgs => has_excess_args = true,
            ParamKind::KeywordOnly => {
                kw_only += 1;
                kwarg_to_index.insert(param_name, i);
            }
            ParamKind::ExcessKwargs => has_excess_kwargs = true,
        }
        if param.has_default() {
            defaults.push(i);
        }
        param_names.push(param_name);
    }
    let param_names = VecLeaker::leak_or_get_once(param_names);

    Ok(Signature {
        param_names,
        kwarg_to_index,
        has_excess_args,
        has_excess_kwargs,
        pos_only,
        pos_or_kw,
        kw_only,
    })
}
