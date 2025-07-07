use std::{
  fs::{DirBuilder, File, ReadDir, create_dir, read_dir, write},
  path::Path,
  rc::Rc,
};

use ego_tree::{NodeId, Tree};
use serde::de::value;
use serde_json::{Map, Number, Value};
use strum::{EnumIs, EnumTryAs};

use crate::{
  HashMap,
  basic::{
    config, database,
    raw_table::{Cell, RawTable},
  },
};
use anyhow::{Result, anyhow, bail};
use error::Error::*;
use std::io::Write;

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

#[derive(Debug, Clone, EnumIs, EnumTryAs)]
pub enum RawData {
  Unknown,
  One(Cell),
  Many,
  Struct(HashMap<Rc<String>, NodeId>),
}

#[derive(Debug)]
pub struct Data {
  pub full_name: String,
  pub typ: usize,
  pub value: Tree<RawData>,
}

impl Data {
  pub fn build_json(&self, database: &Database) -> Result<Value> {
    self.bd_json(database, self.typ, self.value.root().id())
  }

  fn bd_json(&self, database: &Database, typ_id: usize, data_id: NodeId) -> Result<Value> {
    let ty = database.get_type(typ_id).ok_or(类型不存在)?;
    let node = self.value.get(data_id).ok_or(原始数据节点不存在)?;

    let value = match ty {
      Type::Unknown => return Err(类型未知.into()),
      Type::Placeholder(_) => return Err(类型没有定义.into()),
      Type::Dynamic => unimplemented!("目前不支持动态类型"),
      Type::Int => {
        let s = node
          .value()
          .try_as_one_ref()
          .ok_or(原始数据节点类型不匹配)?;
        let v = serde_json::from_str::<Number>(&s)?;
        if v.is_f64() {
          return Err(数字类型错误.into());
        }
        Value::from(v)
      }
      Type::Float => {
        let s = node
          .value()
          .try_as_one_ref()
          .ok_or(原始数据节点类型不匹配)?;
        let v = serde_json::from_str::<Number>(&s)?;
        if !v.is_f64() {
          return Err(数字类型错误.into());
        }
        Value::from(v)
      }
      Type::String => {
        let s = node
          .value()
          .try_as_one_ref()
          .ok_or(原始数据节点类型不匹配)?;
        let v = &***s;
        Value::from(v)
      }
      Type::Bool => {
        let s = node
          .value()
          .try_as_one_ref()
          .ok_or(原始数据节点类型不匹配)?;
        let v = serde_json::from_str::<bool>(&s)?;
        Value::from(v)
      }
      Type::List(tid) => {
        if !node.value().is_many() {
          return Err(原始数据节点类型不匹配.into());
        }
        let mut v = Vec::new();
        for ch in node.children() {
          let item = self.bd_json(database, *tid, ch.id())?;
          v.push(item);
        }
        Value::from(v)
      }
      Type::Dict(key_tid, value_tid) => {
        if !node.value().is_many() {
          return Err(原始数据节点类型不匹配.into());
        }
        let mut v = Map::new();
        for ch in node.children() {
          let mut entry = ch.children();
          let key_id = entry.next().ok_or(原始数据节点类型不匹配)?.id();
          let value_id = entry.next().ok_or(原始数据节点类型不匹配)?.id();
          if entry.next().is_some() {
            return Err(原始数据节点类型不匹配.into());
          }
          let key = self.bd_json(database, *key_tid, key_id)?;
          let value = self.bd_json(database, *value_tid, value_id)?;
          let key_str = serde_json::to_string(&key)?;
          v.insert(key_str, value);
        }
        Value::from(v)
      }
      Type::Struct { full_name, fields } => {
        let fields_data = node
          .value()
          .try_as_struct_ref()
          .ok_or(原始数据节点类型不匹配)?;
        if fields.len() != fields_data.len() {
          return Err(原始数据节点类型不匹配.into());
        }
        let mut v = Map::new();
        for (field_name, f_tid) in fields.iter() {
          let f_id = fields_data.get(field_name).ok_or(原始数据节点类型不匹配)?;
          let field = self.bd_json(database, *f_tid, *f_id)?;
          v.insert(field_name.as_ref().clone(), field);
        }
        Value::from(v)
      }
    };
    Ok(value)
  }
}

#[derive(Debug)]
pub struct Module {
  pub name: String,
  pub type_name_to_id: HashMap<String, usize>,
  pub data: Option<usize>,
  pub children_name_to_id: HashMap<String, NodeId>,
}

impl Module {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      data: None,
      type_name_to_id: HashMap::new(),
      children_name_to_id: HashMap::new(),
    }
  }
}

#[derive(Debug)]
pub struct Database {
  pub types: Vec<Type>,
  pub data: Vec<Data>,
  pub modules: Tree<Module>,
}

impl Database {
  pub fn new() -> Self {
    let mut res = Self {
      types: Vec::new(),
      data: Vec::new(),
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
  pub fn get_data(&self, id: usize) -> Option<&Data> {
    self.data.get(id)
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
  pub fn add_data(&mut self, data: Data) -> usize {
    let mid = self.get_or_create_module(&data.full_name);
    assert!(mid != self.modules.root().id());
    let mut m = self.modules.get_mut(mid).unwrap();
    let data_loc = &mut m.value().data;
    assert!(data_loc.is_none());
    let id = self.data.len();
    *data_loc = Some(id);
    self.data.push(data);
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

  pub fn load_project(&mut self, root: impl AsRef<Path>) -> Result<()> {
    self.ld_project(&root, &root)
  }

  fn ld_project(&mut self, root: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()> {
    for entry in read_dir(path.as_ref())? {
      let entry = entry?;
      if entry.path().is_file() {
        let mut raw_table = RawTable::from_csv(
          entry.path(),
          &config::os_path_to_path(&root, entry.path()).ok_or(文件路径错误)?,
        )?;
        raw_table.build(self)?;
      }else{
        self.ld_project(&root, entry.path())?
      }
    }
    Ok(())
  }

  pub fn generate_data(&self, target: impl AsRef<Path>) -> Result<()> {
    self.gen_data(target, self.modules.root().id())
  }

  fn gen_data(&self, target: impl AsRef<Path>, mid: NodeId) -> Result<()> {
    for ch in self.modules.get(mid).unwrap().children() {
      let path = target.as_ref().join(&ch.value().name);
      if let Some(did) = ch.value().data {
        let data = self.get_data(did).ok_or(数据不存在)?;
        let json = data.build_json(self)?;
        let json_str = serde_json::to_string(&json)?;
        write(path.with_extension("json"), json_str)?;
      } else {
        create_dir(&path)?;
        self.gen_data(&path, ch.id())?;
      }
    }
    Ok(())
  }
}

pub mod error {
  use thiserror::Error;

  #[derive(Debug, Error)]
  pub enum Error {
    #[error("类型不存在")]
    类型不存在,
    #[error("类型未知")]
    类型未知,
    #[error("类型没有定义")]
    类型没有定义,
    #[error("原始数据节点不存在")]
    原始数据节点不存在,
    #[error("原始数据节点类型不匹配")]
    原始数据节点类型不匹配,
    #[error("数字类型错误")]
    数字类型错误,
    #[error("数据不存在")]
    数据不存在,
    #[error("文件路径错误")]
    文件路径错误,
  }
}
