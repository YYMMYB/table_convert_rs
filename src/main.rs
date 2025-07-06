use std::collections::HashMap;

use log::*;

fn main() {
  env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Info)
    .init();

  info!("aaaaa");
  
  let a = "".split('.').collect::<Vec<_>>();
  println!("{:?}",a);
}
