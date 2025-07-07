use std::{
  fs::{create_dir, create_dir_all, write},
  ops::Not,
  path::Path,
};

use anyhow::Result;
use ego_tree::{NodeId, Tree};
use handlebars::{
  Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, handlebars_helper,
  no_escape,
};
use serde_json::{Map, Value};

use crate::basic::database::{Database, Type};

pub struct CSharp<'a> {
  pub reg: Handlebars<'a>,
  pub database: &'a Database,
  pub common_env: Map<String, Value>,
}

impl<'a> CSharp<'a> {
  pub fn new(database: &'a Database) -> Self {
    let mut reg = Handlebars::new();
    reg.register_escape_fn(no_escape);
    reg
      .register_template_file("mod", "./templates/csharp/mod.hbs")
      .unwrap();

    let mut common_env = Map::new();
    common_env.insert(PROJECT_NAMESPACE.to_string(), "Cfg".into());
    common_env.insert("mod_name".to_string(), "Mod".into());
    common_env.insert(
      "mod_usings".to_string(),
      vec!["System", "System.Collections.Generic"].into(),
    );
    common_env.insert("root_namespace".to_string(), "Types".into());

    let mut res = CSharp {
      reg,
      database,
      common_env,
    };

    res
  }
  pub fn generate(&self, target: impl AsRef<Path>) -> Result<()> {
    let root = target
      .as_ref()
      .join(self.common_env["root_namespace"].as_str().unwrap());
    if !root.exists() {
      create_dir_all(root.clone())?;
    }
    self.gene(self.database.modules.root().id(), root)
  }

  fn gene(&self, mid: NodeId, target: impl AsRef<Path>) -> Result<()> {
    if target.as_ref().exists().not() {
      create_dir(target.as_ref())?;
    }

    let mut has_chlid = false;
    let mut data = Vec::new();
    let mut mods = Vec::new();
    for ch in self.database.modules.get(mid).unwrap().children() {
      has_chlid = true;
      let name = ch.value().name.clone();
      dbg!(&name);
      let mut info = Map::new();
      info.insert("name".to_string(), name.into());
      if let Some(did) = ch.value().data {
        let tid = self.database.get_data(did).unwrap().typ;
        let type_name = self.type_name(tid);
        info.insert("type".to_string(), type_name.into());
        data.push(Value::Object(info));
      } else {
        mods.push(Value::Object(info));
      }
    }
    if has_chlid {
      let mut env = Map::new();
      env.insert("mod_namespace".to_string(), self.mod_namespace(mid).into());
      env.insert("data".to_string(), data.into());
      env.insert("mods".to_string(), mods.into());
      env.extend(self.common_env.iter().map(|(k, v)| (k.clone(), v.clone())));
      let mod_content = self.reg.render("mod", &env)?;
      write(
        target
          .as_ref()
          .join(self.common_env["mod_name"].as_str().unwrap())
          .with_added_extension("cs"),
        mod_content,
      )?;

      for ch in self.database.modules.get(mid).unwrap().children() {
        self.gene(ch.id(), target.as_ref().join(&ch.value().name))?;
      }
    }
    Ok(())
  }

  fn type_name(&self, tid: usize) -> String {
    match self.database.get_type(tid).unwrap() {
      Type::Unknown => todo!(),
      Type::Placeholder(_) => todo!(),
      Type::Dynamic => todo!(),
      Type::Int => "int".to_string(),
      Type::Float => "float".to_string(),
      Type::String => "string".to_string(),
      Type::Bool => "bool".to_string(),
      Type::List(pid) => self.type_name(*pid) + "[]",
      Type::Dict(pid1, pid2) => {
        format!(
          "Dictionary<{}, {}>",
          self.type_name(*pid1),
          self.type_name(*pid2)
        )
      }
      Type::Struct { full_name, fields } => {
        code_full_name(&self.common_env[PROJECT_NAMESPACE], full_name)
      }
    }
  }

  fn mod_namespace(&self, mid: NodeId) -> String {
    self.common_env["root_namespace"]
      .as_str()
      .unwrap()
      .to_string()
      + &self.database.module_full_name(mid)
  }
}

fn code_full_name(project_namespace: &Value, full_name: &str) -> String {
  if project_namespace.is_null() {
    full_name[1..].to_string()
  } else {
    project_namespace.as_str().unwrap().to_string() + full_name
  }
}

const NAMESPACE_SEP: &'static str = ".";
const PROJECT_NAMESPACE: &'static str = "project_namespace";
