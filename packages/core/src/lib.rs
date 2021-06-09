#[macro_use]
extern crate napi_derive;

use napi::{CallContext, Env, JsObject, JsUndefined, Result, Task};

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
  exports.create_named_method("rolldown", rolldown)?;
  Ok(())
}

#[derive(Debug)]
struct Rolldown {}

impl Task for Rolldown {
  type Output = ();
  type JsValue = JsUndefined;

  fn compute(&mut self) -> Result<Self::Output> {
    println!("Do nothing");
    Ok(())
  }

  fn resolve(self, env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    env.get_undefined()
  }
}

#[js_function(1)]
fn rolldown(ctx: CallContext) -> Result<JsObject> {
  ctx
    .env
    .spawn(Rolldown {})
    .map(|promise| promise.promise_object())
}
