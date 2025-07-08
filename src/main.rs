use std::{collections::HashMap, env, path::{self, Path, PathBuf}};

use clap::Parser;
use log::*;
use rust_table_export_simple::basic::database::Database;
use anyhow::Result;

fn main() -> Result<()>{
  env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Info)
    .init();
  let args = Args::parse();
  dbg!(&args);
  dbg!(path::absolute(&args.proj));

  let mut db = Database::new();
  db.load_project(args.proj)?;
  db.generate_data(args.data)?;
  db.generate_code(args.code)?;
  Ok(())
}

#[derive(Debug, Parser)]
struct Args {
  #[arg(long, default_value = ".")]
  proj: PathBuf,
  data: PathBuf,
  code: PathBuf,
}