use anyhow::Result;
use csv::ReaderBuilder;
use std::{
  ops::RangeBounds,
  path::{Path, PathBuf},
  rc::Rc,
};

use error::*;
use ndarray::{Array2, ArrayView2, s};

use crate::basic::{
  database::{self, Database, Table},
  parser::Parser,
};

pub type Cell = Rc<String>;

// ╔══════╦══════╗
// ║      ║ head ║
// ║ meta ╠══════╣
// ║      ║ data ║
// ╚══════╩══════╝
//          main = head + data

pub struct RawTable {
  full_name: String,
  storage: Array2<Cell>,
  main_col: usize,
  data_row: usize,
}

impl RawTable {
  pub fn from_csv(path: impl AsRef<Path>, full_name: String) -> Result<Self> {
    let mut rdr = ReaderBuilder::new()
      .has_headers(false)
      .from_path(path.as_ref())?;
    let mut column = 0;
    let mut row = 0;
    let mut cells = Vec::new();
    for record in rdr.records() {
      let record = record?;
      for cell in record.iter() {
        cells.push(Rc::new(cell.to_string()));
      }
      if column == 0 {
        column = record.len();
      }
      row += 1;
    }
    let transpose = cells[0].trim() == "T";
    let storage = if !transpose {
      Array2::from_shape_vec([row, column], cells)?
    } else {
      Array2::from_shape_fn([column, row], |(i, j)| cells[j * column + i].clone())
    };
    Ok(Self {
      full_name,
      storage,
      main_col: 1,
      data_row: 2,
    })
  }

  pub fn get_data_area(&self) -> ArrayView2<Cell> {
    self.storage.slice(s![self.data_row.., self.main_col..])
  }
  pub fn get_head_area(&self) -> ArrayView2<Cell> {
    self.storage.slice(s![..self.data_row, self.main_col..])
  }

  pub fn get_full_name(&self) -> String {
    self.full_name.clone()
  }

  pub fn build_table(&self, database: &mut Database) -> Result<Table> {
    let mut parser = Parser::new();
    let typ = parser.parse_head(self, database)?;
    let value = parser.parse_data(self, database)?;
    Ok(Table {
      full_name: self.full_name.clone(),
      typ,
      value,
    })
  }
}

pub mod error {
  use thiserror::Error;

  #[derive(Debug, Error)]
  pub enum Error {
    #[error(transparent)]
    Any(#[from] anyhow::Error),
    #[error("file_stem 出错")]
    FileStemError,
    #[error("文件名含有非Unicode字符")]
    OsStrError,
  }
}
