use anyhow::Result;
use csv::ReaderBuilder;
use std::{
  ops::RangeBounds,
  path::{Path, PathBuf},
  rc::Rc,
};

use error::*;
use ndarray::{Array2, ArrayView2, s};

use crate::basic::database::Table;

pub type Cell = Rc<String>;
pub struct RawTable {
  name: String,
  storage: Array2<Cell>,
  data: usize,
}

impl RawTable {
  pub fn from_csv(path: impl AsRef<Path>) -> Result<Self> {
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
      name: path
        .as_ref()
        .file_stem()
        .ok_or(Error::FileStemError)?
        .to_str()
        .ok_or(Error::OsStrError)?
        .to_owned(),
      storage,
      data: 2,
    })
  }

  pub fn build_table(&self) -> Table {
    todo!()
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
