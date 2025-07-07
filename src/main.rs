use std::{collections::HashMap, path::Path};

use log::*;

fn main() {
  env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Info)
    .init();

  info!("aaaaa");

  let path = Path::new("D:\\a\\b\\c");
  let rel = path.strip_prefix("D:/a").unwrap();
  println!("{}", rel.to_str().unwrap());

  fn p(path: impl AsRef<Path>) {}
  p("./test/");

  let s = "{}";
  
}
