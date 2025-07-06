use ego_tree::{NodeId, Tree};
use strum::{EnumIs, EnumTryAs};
use anyhow::Result;

use crate::basic::{database::Data, raw_table::{Cell, RawTable}};

#[derive(Debug, EnumIs, EnumTryAs)]
pub enum Formatter {
  Column(Column),
  Struct(Struct),
  Map,
  List,
}

#[derive(Debug)]
pub struct Column {
  pub column: usize,
  pub depth: usize,
  pub field_path: String,
  pub typ: String,
  pub cell: Option<Cell>,
}

#[derive(Debug)]
pub struct Struct {
}

pub struct Parser {
  pub formatters: Tree<Formatter>,
  pub columns: Vec<Vec<NodeId>>,
}

impl Parser {
  pub fn parse_line(&mut self, tb: &RawTable, line: usize, data: &mut Tree<Data>) -> Result<()> {
    
    Ok(())
  }
}
