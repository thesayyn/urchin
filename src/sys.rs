use crate::exec::ExecResult;
use starlark::environment::GlobalsBuilder;
use starlark::eval::Evaluator;
use starlark::values::Value;
use starlark_derive::starlark_module;
use starlark;
use std::process::Command;

#[starlark_module]
pub fn module(builder: &mut GlobalsBuilder) {
    fn exec<'v>(
        #[starlark(require = named)] executable: Value<'v>,
        #[starlark(type = "iter(\"\")", require = named)] arguments: Value<'v>,
        eval: &mut Evaluator<'v, '_>,
    ) -> anyhow::Result<ExecResult> {
        let arguments = Vec::from_iter(
            arguments
                .iterate(eval.heap())?
                .map(|v| v.to_str()),
        );
        let executable = executable.to_str();
        let cmd = Command::new(executable).args(arguments).output().unwrap();

        println!("yo, {:?}", cmd);
        Ok(cmd.into())
    }
}
