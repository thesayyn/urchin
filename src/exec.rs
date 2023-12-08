
use std::fmt;
use std::fmt::Display;
use std::process::Output;
use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::environment::Methods;
use starlark::values::StarlarkValue;
use starlark::{starlark_simple_value, starlark_type};
use starlark_derive::NoSerialize;


#[derive(ProvidesStaticType, Debug, NoSerialize, Allocative)]
pub struct ExecResult {
    stderr: String,
    stdout: String,
    status: i32
}

impl ExecResult {
    pub const TYPE: &'static str = "sys.exec.result";
}

impl StarlarkValue<'_> for ExecResult {
    starlark_type!(ExecResult::TYPE);

    fn get_methods() -> Option<&'static Methods> {
        None
    }
}

starlark_simple_value!(ExecResult);

impl Display for ExecResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sys.exec.result(\nstatus = {}\nstdout = {}\nstderr = {}\n)", &self.status, &self.stdout, &self.stderr)
    }
}

impl Into<ExecResult> for Output {
    fn into(self) -> ExecResult {
        ExecResult {
            stdout: String::from_utf8_lossy(&self.stdout).to_string(),
            stderr: String::from_utf8_lossy(&self.stderr).to_string(),
            status: self.status.code().unwrap()
        }
    }
}