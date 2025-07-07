use crate::HashMap;
use anyhow::Result;
use std::{
  cell::{LazyCell, OnceCell},
  hash::Hash,
  path::Path,
  sync::LazyLock,
};

pub const PATH_SPLITOR: char = '.';

pub fn path_components(path: &str) -> Vec<&str> {
  path.split(PATH_SPLITOR).collect()
}

pub fn path_join(comp: &[&str]) -> String {
  comp.join(&PATH_SPLITOR.to_string())
}

pub fn path_parent(path: &str) -> &str {
  if let Some((p, _)) = path.rsplit_once(PATH_SPLITOR) {
    p
  } else {
    ""
  }
}
pub fn path_name(path: &str) -> &str {
  if let Some((_, n)) = path.rsplit_once(PATH_SPLITOR) {
    n
  } else {
    path
  }
}

pub fn path_rel_to_global(rel_path: &str) -> String {
  path_join(&["", rel_path])
}

pub const GENERIC_SPLITOR: char = '-';
pub fn generic_type_name(base: &str, type_params: &[usize]) -> String {
  let mut v = vec![base.to_string()];
  v.extend(type_params.iter().map(|i| i.to_string()));
  v.join(&GENERIC_SPLITOR.to_string())
}

pub const ITEM_POSTFIX: &str = "_item";
pub fn table_item_type_full_name(table_full_name: &str) -> String {
  let mod_name = path_components(&table_full_name)
    .last()
    .unwrap()
    .to_string();
  let item_name = mod_name + ITEM_POSTFIX;
  let full_name = path_join(&[&table_full_name, &item_name]);
  full_name
}

pub static BUILTIN_TYPE_NAMES: LazyLock<HashMap<&'static str, &'static str>> =
  LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("i", "int");
    map.insert("f", "float");
    map.insert("s", "string");
    map.insert("b", "bool");
    map.insert("l", "list");
    map.insert("d", "dictionary");
    map
  });

pub const TYPE_PARAMETER_DELIMINATOR_LEFT: &str = "<";
pub const TYPE_PARAMETER_DELIMINATOR_RIGHT: &str = ">";
pub const TYPE_PARAMETER_SPLITOR: &str = ",";

pub fn os_path_to_path(
  root_os_path: impl AsRef<Path>,
  os_path: impl AsRef<Path>,
) -> Option<String> {
  Some(path_rel_to_global(
    &os_path
      .as_ref()
      .strip_prefix(root_os_path)
      .ok()?
      .with_extension("")
      .to_str()?
      .replace(['/', '\\'], "."),
  ))
}

#[cfg(test)]
mod test {
  use crate::basic::config::os_path_to_path;

  #[test]
  fn test_os_path_to_path() {
    let name = os_path_to_path("D:\\a\\b\\c", "D:\\a\\b\\c\\d\\e.csv");
    assert_eq!(name, Some(".d.e".to_string()));
    let name = os_path_to_path("D:\\a\\b\\c", "D:\\a\\b\\c\\d\\e");
    assert_eq!(name, Some(".d.e".to_string()));
  }
}
