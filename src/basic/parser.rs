use std::{ops::Not, rc::Rc};

use anyhow::Result;
use ego_tree::{NodeId, Tree};
use strum::{EnumIs, EnumTryAs};

use crate::{
  HashMap,
  basic::{
    config,
    database::{self, Data, Database, ItemTag, Type},
    raw_table::{Cell, RawTable},
  },
};

pub struct Column {
  pub field: Rc<String>,
  pub typ: usize,
}
pub struct Parser {
  pub columns: Vec<Column>,
}

impl Parser {
  pub fn new() -> Self {
    Self { columns: Vec::new() }
  }
  pub fn parse_head(&mut self, raw_table: &RawTable, database: &mut Database) -> Result<usize> {
    let mut fields = HashMap::new();
    let head_area = raw_table.get_head_area();
    let mut last_field: Option<Rc<String>> = None;
    let mut last_type = None;
    for c in 0..head_area.shape()[1] {
      let raw_field = head_area.get([0, c]).unwrap();
      let raw_type = head_area.get([1, c]).unwrap();
      let field;
      let typ;
      if raw_field.trim().is_empty() {
        field = last_field.clone().unwrap();
        typ = last_type.unwrap();
      } else {
        field = Rc::new(raw_field.trim().to_string());
        last_field = Some(field.clone());
        typ = parse_raw_type(raw_type, database)?;
        last_type = Some(typ.clone());
        fields.insert(field.clone(), typ.clone());
      }
      self.columns.push(Column {
        field: field.clone(),
        typ: typ.clone(),
      });
    }
    let item = Type::Struct {
      full_name: config::table_item_type_full_name(&raw_table.get_full_name()),
      fields,
    };
    let iid = database.add_type(item);
    let did = database.add_type(Type::Dict(self.columns[0].typ, iid));
    Ok(did)
  }
  pub fn parse_data(&self, raw_table: &RawTable, database: &Database) -> Result<Tree<Data>> {
    let data_area = raw_table.get_data_area();
    let mut data_tree = Tree::new(Data::Many);
    for row in 0..data_area.shape()[0] {
      let mut root_mut = data_tree.root_mut();
      let item_id = root_mut.append(Data::Map(HashMap::new())).id();
      for col in 0..data_area.shape()[1] {
        let cell = data_area.get([row, col]).unwrap();
        let field = &self.columns[col].field;
        if let Some(id) = data_tree
          .get(item_id)
          .unwrap()
          .value()
          .try_as_map_ref()
          .unwrap()
          .get(field)
        {
          data_tree
            .get_mut(*id)
            .ok_or(error::Error::Map存储了无效子节点)?
            .append(Data::One(cell.clone()));
        } else {
          let typ = database
            .get_type(self.columns[col].typ)
            .ok_or(error::Error::类型不存在)?;
          assert!(!typ.is_placeholder());
          let id;
          if typ.is_list() {
            let mut item = data_tree.get_mut(item_id).unwrap();
            let mut arr = item.append(Data::Many);
            id = arr.id();
            arr.append(Data::One(cell.clone())).id();
          } else {
            id = data_tree
              .get_mut(item_id)
              .unwrap()
              .append(Data::One(cell.clone()))
              .id();
          }
          data_tree
            .get_mut(item_id)
            .unwrap()
            .value()
            .try_as_map_mut()
            .unwrap()
            .insert(field.clone(), id);
        }
      }
    }
    Ok(data_tree)
  }
}

pub fn parse_raw_type(raw_type: &str, database: &mut Database) -> Result<usize> {
  let raw_type = raw_type.trim();
  let tid = match raw_type {
    "i" | "f" | "s" | "b" => database
      .get_type_id_by_full_name(&config::path_rel_to_global(
        config::BUILTIN_TYPE_NAMES.get(raw_type).unwrap(),
      ))
      .unwrap(),
    _ => {
      let a = raw_type
        .chars()
        .next()
        .ok_or(error::Error::类型声明语法错误)?;
      match a {
        'l' => {
          let follow = raw_type[1..].trim();
          if follow.starts_with(config::TYPE_PARAMETER_DELIMINATOR_LEFT)
            && follow.ends_with(config::TYPE_PARAMETER_DELIMINATOR_RIGHT)
          {
            let mut pars = follow[1..follow.len() - 1].split(config::TYPE_PARAMETER_SPLITOR);
            let p1 = pars.next().ok_or(error::Error::类型声明语法错误)?;
            if let Some(p) = pars.next() {
              if !p.trim().is_empty() {
                return Err(error::Error::类型声明语法错误.into());
              } else if pars.next().is_some() {
                return Err(error::Error::类型声明语法错误.into());
              }
            };
            let pid = parse_raw_type(p1, database)?;
            database.add_type(Type::List(pid))
          } else {
            return Err(error::Error::类型声明语法错误.into());
          }
        }
        'd' => {
          let follow = raw_type[1..].trim();
          if follow.starts_with(config::TYPE_PARAMETER_DELIMINATOR_LEFT)
            && follow.ends_with(config::TYPE_PARAMETER_DELIMINATOR_RIGHT)
          {
            let mut pars = follow[1..follow.len() - 1].split(config::TYPE_PARAMETER_SPLITOR);
            let p1 = pars.next().ok_or(error::Error::类型声明语法错误)?;
            let p2 = pars.next().ok_or(error::Error::类型声明语法错误)?;
            if let Some(p) = pars.next() {
              if !p.trim().is_empty() {
                return Err(error::Error::类型声明语法错误.into());
              } else if pars.next().is_some() {
                return Err(error::Error::类型声明语法错误.into());
              }
            };
            let p1id = parse_raw_type(&p1, database)?;
            let p2id = parse_raw_type(&p2, database)?;
            database.add_type(Type::Dict(p1id, p2id))
          } else {
            return Err(error::Error::类型声明语法错误.into());
          }
        }
        _ => {
          unimplemented!("目前不支持自定义类型")
        }
      }
    }
  };
  Ok(tid)
}

pub mod error {
  use thiserror::Error;

  #[derive(Debug, Error)]
  pub enum Error {
    #[error("Map存储了无效子节点")]
    Map存储了无效子节点,
    #[error("类型不存在")]
    类型不存在,
    #[error("类型声明语法错误")]
    类型声明语法错误,
  }
}
