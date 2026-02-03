use pyo3::{exceptions::PyValueError, prelude::*};
use tracing::Level;

use crate::{
    callsite::Context,
    event::{self, ErrCallsite, RetCallsite, YieldCallsite},
};

#[derive(Default, Clone)]
pub enum RetLog {
    RetYieldErr,
    RetErr,
    Err,
    #[default]
    None,
}

impl RetLog {
    pub fn from_opts(ret: bool, no_yield: bool, err_only: bool) -> PyResult<Self> {
        match (ret, no_yield, err_only) {
            // err_only implies no_yield & !ret
            (true, true, true) => Err(PyValueError::new_err(
                "`err_only` cannot be used with `ret` or `no_yield`",
            )),
            (true, true, false) => Ok(Self::RetErr),
            // err_only implies !ret
            (true, false, true) => Err(PyValueError::new_err(
                "`err_only` cannot be used with `ret`",
            )),
            (true, false, false) => Ok(Self::RetYieldErr),
            // err_only implies no_yield
            (false, true, true) => Err(PyValueError::new_err(
                "`err_only` cannot be used with `no_yield`",
            )),
            // !ret implies no_yield
            (false, true, false) => Err(PyValueError::new_err(
                "`no_yield` cannot be used without `ret`",
            )),
            (false, false, true) => Ok(Self::Err),
            (false, false, false) => Ok(Self::None),
        }
    }

    fn ret(&self) -> bool {
        match self {
            Self::RetYieldErr | Self::RetErr => true,
            Self::Err | Self::None => false,
        }
    }

    fn r#yield(&self) -> bool {
        match self {
            Self::RetYieldErr => true,
            Self::RetErr | Self::Err | Self::None => false,
        }
    }

    fn r#err(&self) -> bool {
        match self {
            Self::RetYieldErr | Self::RetErr | Self::Err => true,
            Self::None => false,
        }
    }

    pub fn r#callsites(
        &self,
        ctx: Context<'_>,
        level: Level,
    ) -> (
        Option<RetCallsite>,
        Option<YieldCallsite>,
        Option<ErrCallsite>,
    ) {
        let ret_callsite = self.ret().then(|| event::ret_callsite(ctx.clone(), level));
        let yield_callsite = self
            .r#yield()
            .then(|| event::yield_callsite(ctx.clone(), level));
        let err_callsite = self.err().then(|| event::err_callsite(ctx, level));
        (ret_callsite, yield_callsite, err_callsite)
    }
}
