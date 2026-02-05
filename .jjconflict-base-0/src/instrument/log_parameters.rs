use pyo3::{exceptions::PyValueError, prelude::*};
use tracing::Level;

use crate::{
    callsite::Context,
    event::{self, ErrCallsite, RetCallsite, YieldCallsite},
};

pub const DEFAULT_LEVEL: Level = Level::INFO;

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
        levels: &LevelOverrides,
    ) -> (
        Option<RetCallsite>,
        Option<YieldCallsite>,
        Option<ErrCallsite>,
    ) {
        let ret_callsite = self
            .ret()
            .then(|| event::ret_callsite(ctx.clone(), levels.ret()));
        let yield_callsite = self
            .r#yield()
            .then(|| event::yield_callsite(ctx.clone(), levels.r#yield()));
        let err_callsite = self.err().then(|| event::err_callsite(ctx, levels.err()));
        (ret_callsite, yield_callsite, err_callsite)
    }
}

#[derive(Clone)]
pub struct LevelOverrides {
    pub default: Level,
    pub ret: Option<Level>,
    pub err: Option<Level>,
    pub r#yield: Option<Level>,
}

impl LevelOverrides {
    fn get_or_default(&self, r#override: Option<Level>) -> Level {
        r#override.unwrap_or(self.default)
    }

    pub fn span(&self) -> Level {
        self.default
    }

    fn ret(&self) -> Level {
        self.get_or_default(self.ret)
    }

    fn err(&self) -> Level {
        self.get_or_default(self.err)
    }

    fn r#yield(&self) -> Level {
        self.get_or_default(self.r#yield)
    }
}

impl Default for LevelOverrides {
    fn default() -> Self {
        Self {
            default: DEFAULT_LEVEL,
            ret: None,
            err: None,
            r#yield: None,
        }
    }
}
