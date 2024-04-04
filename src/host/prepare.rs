use core::fmt;
use std::fmt::Display;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::collections::SmallMap;
use starlark::starlark_complex_value;
use starlark::values::{StarlarkValue, ValueLike};
use starlark_derive::{starlark_value, Coerce, Freeze, NoSerialize, Trace};

#[derive(Debug, Default, Trace, Freeze, Coerce, ProvidesStaticType, NoSerialize, Allocative)]
#[repr(C)]
pub struct PrepareContextGen<V> {
    pub queries: SmallMap<String, V>,
    pub files: V,
}

starlark_complex_value!(pub PrepareContext);

impl<'v,V: ValueLike<'v>> Display for PrepareContextGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<prepare_ctx unknown>")
    }
}

#[starlark_value(type = "PrepareContext")]
impl<'v, V: ValueLike<'v> + 'v> StarlarkValue<'v> for PrepareContextGen<V> where
    Self: ProvidesStaticType<'v>
{
   
}
