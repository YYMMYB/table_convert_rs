use ego_tree::{NodeId, Tree};
use serde_json::Value;

use crate::{HashMap, basic::raw_table::Cell};

pub enum ItemTag {
  ID(usize),
  FullName(String),
  RelName(String),
}

pub enum Type {
  Int,
  Float,
  String,
  Bool,
  Struct {
    full_name: String,
    fields: HashMap<String, ItemTag>,
  },
  List(Box<Type>),
  Map(Box<Type>, Box<Type>),
}

pub enum Data {
  Unknown,
  One(Cell),
  Many,
  Map(HashMap<String, NodeId>),
}

pub struct Table {
  pub full_name: String,
  pub typ: ItemTag,
  pub value: Tree<Data>,
}

pub struct Module {
  pub type_name_to_id: HashMap<String, usize>,
  pub table_name_to_id: HashMap<String, usize>,
  pub children_name_to_id: HashMap<String, NodeId>,
}

pub struct Database {
  pub types: Vec<Type>,
  pub tables: Vec<Table>,
  pub modules: Tree<Module>,
}
