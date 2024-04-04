use anyhow::Context;
use fides::host::{
    configure::configure_globals,
    extension::{Extension, ExtensionCollection},
    prepare::PrepareContext,
};
use starlark::{
    environment::{GlobalsBuilder, Module},
    eval::Evaluator,
    syntax::{AstModule, Dialect},
    values::{ValueLike, AllocValue},
};
use std::path::Path;

fn run_extension<'v>() -> anyhow::Result<()> {
    let module = Module::new();
    let globals = GlobalsBuilder::standard().with(configure_globals).build();
    let mut eval = Evaluator::new(&module);
    let ast_module =
        AstModule::parse_file(&Path::new("examples/javascript.star"), &Dialect::Extended)?;
    module.set_extra_value(
        eval.heap()
            .alloc_complex_no_freeze(ExtensionCollection::default()),
    );
    eval.eval_module(ast_module, &globals)?;
    let collection = module
        .extra_value()
        .context("extra value should be set")?
        .downcast_ref::<ExtensionCollection>()
        .context("failed to cast")?;
    let collection = collection.collection.lock().expect("");
    let (_, first_extension) = collection.first().unwrap();
    let extension = first_extension.downcast_ref::<Extension>().unwrap();

    let prepare_ctx = PrepareContext::default();
    let prepare_val = eval.eval_function(extension.prepare, &[prepare_ctx.alloc_value(eval.heap())], &[]);
    println!("{:?}", prepare_val);
    Ok(())
}

fn main() {
    run_extension().expect("failed");
}
