use allocative::Allocative;
use derive_more::Display;
use starlark::starlark_simple_value;
use starlark::values::dict::DictOf;
use starlark::values::StarlarkValue;
use starlark_derive::{starlark_value, NoSerialize, Trace, ProvidesStaticType};

#[derive(ProvidesStaticType, Trace, Allocative, Debug, Display, NoSerialize, Clone)]
#[display(fmt = "{}", "self.expr")]
pub struct Query {
    pub extensions: Vec<String>,
    pub parser: String,
    pub expr: String,
}

#[starlark_value(type = "Query")]
impl<'v> StarlarkValue<'v> for Query {}

starlark_simple_value!(Query);


#[derive(ProvidesStaticType, Trace, Allocative, Debug, Display, NoSerialize, Clone)]
#[display(fmt = "{:#?}, {:#?}", "self.extensions", "self.queries")]
pub struct Metadata<'v> {
    pub extensions: Vec<String>,
    pub queries: DictOf<'v, String, Query>,
}
#[starlark_value(type = "Metadata")]
impl<'v> StarlarkValue<'v> for Metadata<'v> {}

starlark_simple_value!(Metadata);