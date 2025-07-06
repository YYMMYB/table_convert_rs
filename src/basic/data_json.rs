use anyhow::Result;
use serde_json::{Map, Number, Value};

fn a() -> Result<()> {
  let mut v = Value::Array(vec![Value::Null, Value::from(true), Value::from(123)]);
  let j = v.to_string();
  Ok(())
}
