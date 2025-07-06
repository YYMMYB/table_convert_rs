use std::collections::HashMap;

use log::*;

fn main() {
  env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Info)
    .init();

  info!("aaaaa");
  use anyhow::Result;
  use serde_json::{Map, Number, Value};
}
