use std::rc::Rc;

use ego_tree::{NodeId, Tree};
use serde_json::Value;
use strum::{EnumIs, EnumTryAs};

use crate::{
  HashMap,
  basic::{config, database, raw_table::Cell},
};
use anyhow::Result;

#[derive(Debug, Clone, EnumIs, EnumTryAs)]
pub enum ItemTag {
  ID(usize),
  FullName(String),
  RelName(String),
}

impl ItemTag {
  pub fn to_id(&self, database: &Database, module: &str) -> Option<Self> {
    match self {
      ItemTag::ID(id) => Some(ItemTag::ID(*id)),
      ItemTag::FullName(full_name) => {
        let id = database.get_type_id_by_full_name(full_name)?;
        Some(ItemTag::ID(id))
      }
      ItemTag::RelName(name) => {
        let full_name = config::path_join(&[module, name]);
        let id = database.get_type_id_by_full_name(&full_name)?;
        Some(ItemTag::ID(id))
      }
    }
  }
}

#[derive(Debug, Clone, EnumIs, EnumTryAs)]
pub enum Type {
  Unknown,
  Placeholder(String),
  Dynamic,
  Int,
  Float,
  String,
  Bool,
  List(usize),
  Dict(usize, usize),
  Struct {
    full_name: String,
    fields: HashMap<Rc<String>, usize>,
  },
}

impl Type {
  /// 要求: (否则panic)
  ///
  /// 不能是 [Type::Unknown] 或 [Type::Placeholder]

  fn get_full_name(&self) -> String {
    match self {
      Type::Unknown | Type::Placeholder(_) => panic!("未知类型"),
      Type::Dynamic => ".dynamic".to_string(),
      Type::Int => ".int".to_string(),
      Type::Float => ".float".to_string(),
      Type::String => ".string".to_string(),
      Type::Bool => ".bool".to_string(),
      &Type::List(item_tag) => {
        let id = item_tag;
        ".".to_string() + &config::generic_type_name("list", &[id])
      }
      &Type::Dict(key_tag, value_tag) => {
        let id = key_tag;
        let id2 = value_tag;
        ".".to_string() + &config::generic_type_name("dictionary", &[id, id2])
      }
      Type::Struct { full_name, .. } => full_name.clone(),
    }
  }
}

#[derive(Debug, EnumIs, EnumTryAs)]
pub enum Data {
  Unknown,
  One(Cell),
  Many,
  Map(HashMap<Rc<String>, NodeId>),
}

#[derive(Debug)]
pub struct Table {
  pub full_name: String,
  pub typ: usize,
  pub value: Tree<Data>,
}

#[derive(Debug)]
pub struct Module {
  pub name: String,
  pub type_name_to_id: HashMap<String, usize>,
  pub table_name_to_id: HashMap<String, usize>,
  pub children_name_to_id: HashMap<String, NodeId>,
}

impl Module {
  pub fn new(name:&str) -> Self {
    Self {
      name: name.to_string(),
      type_name_to_id: HashMap::new(),
      table_name_to_id: HashMap::new(),
      children_name_to_id: HashMap::new(),
    }
  }
}

#[derive(Debug)]
pub struct Database {
  pub types: Vec<Type>,
  pub tables: Vec<Table>,
  pub modules: Tree<Module>,
}

impl Database {
  pub fn new() -> Self {
    let mut res = Self {
      types: Vec::new(),
      tables: Vec::new(),
      modules: Tree::new(Module::new("")),
    };
    res.add_type(Type::Int);
    res.add_type(Type::Float);
    res.add_type(Type::String);
    res.add_type(Type::Bool);
    res
  }
  pub fn get_or_create_module(&mut self, module: &str) -> NodeId {
    let mods = config::path_components(module);
    assert!(mods[0] == "");
    let mut mid = self.modules.root().id();
    for &mod_name in &mods[1..] {
      if let Some(&id) = self
        .modules
        .get(mid)
        .unwrap()
        .value()
        .children_name_to_id
        .get(mod_name)
      {
        mid = id;
      } else {
        let mut node = self.modules.get_mut(mid).unwrap();
        let id = node.append(Module::new(mod_name)).id();
        node
          .value()
          .children_name_to_id
          .insert(mod_name.to_string(), id);
        mid = id;
      }
    }
    mid
  }
  pub fn get_type(&self, id: usize) -> Option<&Type> {
    self.types.get(id)
  }
  pub fn get_table(&self, id: usize) -> Option<&Table> {
    self.tables.get(id)
  }
  pub fn get_module(&self, module: &str) -> Option<NodeId> {
    let mods = config::path_components(module);
    assert!(mods[0] == "");
    let mut m = self.modules.root();
    for &mod_name in &mods[1..] {
      let id = m.value().children_name_to_id.get(mod_name)?.to_owned();
      m = self.modules.get(id)?;
    }
    Some(m.id())
  }
  pub fn add_type(&mut self, ty: Type) -> usize {
    let mid = if let Type::Struct { full_name, .. } = &ty {
      self.get_or_create_module(config::path_parent(&full_name))
    } else {
      self.modules.root().id()
    };

    let id = self.types.len();
    self
      .modules
      .get_mut(mid)
      .unwrap()
      .value()
      .type_name_to_id
      .insert(config::path_name(&ty.get_full_name()).to_string(), id);
    self.types.push(ty);
    id
  }

  pub fn get_type_id_by_full_name(&self, name: &str) -> Option<usize> {
    let mid = self.get_module(config::path_parent(name))?;
    self
      .modules
      .get(mid)?
      .value()
      .type_name_to_id
      .get(config::path_name(name))
      .copied()
  }
}
