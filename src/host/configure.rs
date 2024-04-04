use starlark::{
    environment::GlobalsBuilder,
    eval::Evaluator,
    values::{dict::DictOf, none::NoneType, Value, ValueLike},
};
use starlark_derive::starlark_module;

use crate::host::{
    extension::{Extension, ExtensionCollection},
    types::{Metadata, Query},
};

pub fn configure_globals(g: &mut GlobalsBuilder) {
    g.struct_("configure", global);
}

#[starlark_module]
fn global(g: &mut GlobalsBuilder) {
    #[allow(non_snake_case)]
    fn Metadata<'v>(
        extensions: Vec<String>,
        queries: DictOf<'v, String, Query>,
    ) -> anyhow::Result<Metadata> {
        Ok(Metadata {
            extensions: extensions,
            queries: queries,
        })
    }

    #[allow(non_snake_case)]
    fn Query<'v>(
        extensions: Vec<String>,
        parser: Value<'v>,
        expr: Value<'v>
    ) -> anyhow::Result<Query> {
        Ok(Query {
            extensions: extensions,
            parser: parser,
            expr: expr,
        })
    }

    #[allow(non_snake_case)]
    fn Extension<'v>(
        name: Value<'v>,
        prepare: Value<'v>,
        declare_imports: Value<'v>,
        declare_exports: Value<'v>,
        eval: &mut Evaluator<'v, '_>,
    ) -> anyhow::Result<NoneType> {
        let extension = Extension {
            declare_exports,
            declare_imports,
            prepare,
            name,
        };
        let extra = eval
            .module()
            .extra_value()
            .unwrap()
            .downcast_ref_err::<ExtensionCollection>()?;
        dbg!(format!("Registering {}", name.to_str()));
        let mut m = extra.collection.lock().unwrap();

        m.insert(
            name.to_string(),
            eval.heap().alloc_complex(extension),
        );
        println!("{}", extra);
        Ok(NoneType)
    }
}
