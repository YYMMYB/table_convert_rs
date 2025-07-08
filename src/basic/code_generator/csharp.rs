use std::{
  fs::{create_dir, create_dir_all, write},
  ops::Not,
  path::Path,
};

use anyhow::Result;
use ego_tree::{NodeId, Tree};
use handlebars::{Handlebars, no_escape};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::basic::{
  config::{self, OptionJoin},
  database::{Database, Type},
};

pub struct CSharp<'a> {
  pub reg: Handlebars<'a>,
  pub database: &'a Database,
  pub common_env: CommonEnv,
}

impl<'a> CSharp<'a> {
  pub fn try_new(database: &'a Database) -> Result<Self> {
    let mut reg = Handlebars::new();
    reg.register_escape_fn(no_escape);
    reg.register_template_file("mod", "./templates/csharp/mod.hbs")?;
    reg.register_template_file("class", "./templates/csharp/class.hbs")?;
    reg.register_template_file("util", "./templates/csharp/util.hbs")?;
    reg.register_template_file("access", "./templates/csharp/access.hbs")?;

    let project_namespace = Some("Cfg");
    let common_namespace_name = "Common";
    let mut res = CSharp {
      reg,
      database,
      common_env: CommonEnv {
        project_namespace: project_namespace.map(|s| s.to_string()),
        root_namespace: "Types".to_string(),
        mod_class_name: "Mod".to_string(),
        mod_usings: vec![
          "System".to_string(),
          "System.Collections.Generic".to_string(),
          "System.Text.Json.Serialization".to_string(),
        ],
        class_usings: vec![
          "System".to_string(),
          "System.Collections.Generic".to_string(),
          "System.Text.Json.Serialization".to_string(),
        ],
        common_namespace_name: common_namespace_name.to_string(),
        common_namespace: [project_namespace, Some(common_namespace_name)].option_join(NAMESPACE_SEPARATOR),
      },
    };

    Ok(res)
  }
  pub fn generate(&self, target: impl AsRef<Path>) -> Result<()> {
    let common = target.as_ref().join(self.common_env.common_namespace_name.as_str());
    if !common.exists() {
      create_dir_all(common.clone())?;
    }
    let content = self.reg.render("util", &self.common_env, )?;
    write(common.join("Util.cs"), content)?;
    let content = self.reg.render("access",&self.common_env)?;
    write(common.join("IDataAccess.cs"), content)?;
    let root = target
      .as_ref()
      .join(self.common_env.root_namespace.as_str());
    if !root.exists() {
      create_dir_all(root.clone())?;
    }
    self.gen_mods(self.database.modules.root().id(), root);
    Ok(())
  }

  fn gen_mods(&self, mid: NodeId, target: impl AsRef<Path>) -> Result<()> {
    if target.as_ref().exists().not() {
      create_dir(target.as_ref())?;
    }
    let mod_namespace = self.mod_namespace(mid);

    // 子模块
    let mut has_chlid = false;
    let mut data = Vec::new();
    let mut mods = Vec::new();
    for ch in self.database.modules.get(mid).unwrap().children() {
      has_chlid = true;
      let name = ch.value().name.clone();
      dbg!(&name);
      if let Some(did) = ch.value().data {
        let tid = self.database.get_data(did).unwrap().typ;
        let type_full_name = self.type_full_name(tid);
        let fenv = DataFieldEnv {
          name: name.clone(),
          type_full_name,
          data_file_name: name.clone() + ".json",
        };
        data.push(fenv);
      } else {
        let fenv = SubmoduleFieldEnv {
          name: name.clone(),
          namespace: self.mod_namespace(ch.id()),
          data_folder_name: name.clone(),
        };
        mods.push(fenv);
      }
    }
    if has_chlid {
      let mut env = ModuleFileEnv {
        common_env: &self.common_env,
        mod_namespace: mod_namespace.clone(),
        data_fields: data,
        submodule_fields: mods,
      };
      let mod_content = self.reg.render("mod", &env)?;
      write(
        target
          .as_ref()
          .join(self.common_env.mod_class_name.as_str())
          .with_added_extension("cs"),
        mod_content,
      )?;

      for ch in self.database.modules.get(mid).unwrap().children() {
        self.gen_mods(ch.id(), target.as_ref().join(&ch.value().name))?;
      }
    }

    // 包含的类型
    for (name, &id) in self
      .database
      .modules
      .get(mid)
      .unwrap()
      .value()
      .type_name_to_id
      .iter()
    {
      let ty = self.database.get_type(id).unwrap();
      match ty {
        Type::Struct { full_name, fields } => {
          let name = self.named_type_name(&full_name);

          let mut field_envs = Vec::new();
          for (fname, ftid) in fields.iter() {
            let ftyname = self.type_full_name(*ftid);
            let fenv = ClassFieldEnv {
              name: fname.as_ref().clone(),
              type_full_name: ftyname,
            };
            field_envs.push(fenv);
          }
          let env = ClassFileEnv {
            common_env: &self.common_env,
            namespace: mod_namespace.clone(),
            name: name.clone(),
            fields: field_envs,
          };

          let content = self.reg.render("class", &env)?;
          write(
            target.as_ref().join(name).with_added_extension("cs"),
            content,
          )?;
        }
        _ => {}
      }
    }
    Ok(())
  }

  fn type_full_name(&self, tid: usize) -> String {
    match self.database.get_type(tid).unwrap() {
      Type::Unknown => todo!(),
      Type::Placeholder(_) => todo!(),
      Type::Dynamic => todo!(),
      Type::Int => "int".to_string(),
      Type::Float => "float".to_string(),
      Type::String => "string".to_string(),
      Type::Bool => "bool".to_string(),
      Type::List(pid) => self.type_full_name(*pid) + "[]",
      Type::Dict(pid1, pid2) => {
        format!(
          "Dictionary<{}, {}>",
          self.type_full_name(*pid1),
          self.type_full_name(*pid2)
        )
      }
      Type::Struct { full_name, fields } => self.named_type_full_name(&full_name),
    }
  }

  fn mod_namespace(&self, mid: NodeId) -> String {
    let nm = self.common_env.root_namespace.clone() + &self.database.module_full_name(mid);
    self.full_name(&nm)
  }

  fn named_type_full_name(&self, engine_full_name: &str) -> String {
    let nm = self.common_env.root_namespace.clone() + engine_full_name;
    self.full_name(&nm)
  }

  fn full_name(&self, rel_proj_name: &str) -> String {
    if let Some(pns) = &self.common_env.project_namespace {
      return [&**pns, rel_proj_name].join(NAMESPACE_SEPARATOR);
    } else {
      return rel_proj_name.to_string();
    }
  }

  fn named_type_name(&self, engine_full_name: &str) -> String {
    config::path_name(engine_full_name).to_string()
  }
}

#[derive(Debug, Serialize)]
pub struct CommonEnv {
  pub project_namespace: Option<String>,
  pub root_namespace: String,
  pub mod_class_name: String,
  pub mod_usings: Vec<String>,
  pub class_usings: Vec<String>,
  pub common_namespace_name: String,
  pub common_namespace: String,
}
#[derive(Debug, Serialize)]
pub struct ModuleFileEnv<'a> {
  #[serde(flatten)]
  pub common_env: &'a CommonEnv,
  pub mod_namespace: String,
  pub data_fields: Vec<DataFieldEnv>,
  pub submodule_fields: Vec<SubmoduleFieldEnv>,
}

#[derive(Debug, Serialize)]
pub struct DataFieldEnv {
  pub name: String,
  pub type_full_name: String,
  pub data_file_name: String,
}

#[derive(Debug, Serialize)]
pub struct SubmoduleFieldEnv {
  pub name: String,
  pub namespace: String,
  pub data_folder_name: String,
}

#[derive(Debug, Serialize)]
pub struct ClassFileEnv<'a> {
  #[serde(flatten)]
  pub common_env: &'a CommonEnv,
  pub namespace: String,
  pub name: String,
  pub fields: Vec<ClassFieldEnv>,
}

#[derive(Debug, Serialize)]
pub struct ClassFieldEnv {
  pub name: String,
  pub type_full_name: String,
}

const NAMESPACE_SEPARATOR: &'static str = ".";
const PROJECT_NAMESPACE: &'static str = "project_namespace";
