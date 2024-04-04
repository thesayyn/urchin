use core::fmt;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::collections::SmallMap;
use starlark::starlark_complex_value;
use starlark::values::{StarlarkValue, Value, ValueLike};
use starlark_derive::{starlark_value, Coerce, Freeze, NoSerialize, Trace};

#[derive(Debug, Trace, Coerce, Freeze, ProvidesStaticType, NoSerialize, Allocative)]
#[repr(C)]
pub struct ExtensionGen<V> {
    pub name: V,
    #[allocative(skip)]
    pub prepare: V,
    #[allocative(skip)]
    pub declare_imports: V,
    #[allocative(skip)]
    pub declare_exports: V,
}

starlark_complex_value!(pub Extension);

impl<V: Display> Display for ExtensionGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<extension unknown")?;
        write!(f, ">")
    }
}

#[starlark_value(type = "Extension")]
impl<'v, V: ValueLike<'v> + 'v> StarlarkValue<'v> for ExtensionGen<V> where
    Self: ProvidesStaticType<'v>
{
   
}

#[derive(
    ProvidesStaticType, Default, Trace, Debug, derive_more::Display, NoSerialize, Allocative,
)]
#[display(fmt = "{:?}", self)]
pub struct ExtensionCollection<'v> {
    pub collection: Arc<Mutex<SmallMap<String, Value<'v>>>>,
}

#[starlark_value(type = "ExtensionCollection")]
impl<'v> StarlarkValue<'v> for ExtensionCollection<'v> {

}
