use std::{collections::HashMap, path::Path};

use log::*;

fn main() {
  env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Info)
    .init();

  info!("aaaaa");

  let d = Dt {
    a: Some("aaaaa".to_string()),
    b: None,
    e: Some(E::Y {
      y: "y".to_string(),
      yy: 3.14,
    }),
  };
  let j = serde_json::to_string(&d).unwrap();
  dbg!(&j);
}
use serde::*;
#[derive(Debug, Serialize)]
pub struct Dt {
  pub a: Option<String>,
  pub b: Option<String>,
  pub e: Option<E>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum E {
  X(i32),
  Y { y: String, yy: f32 },
}
