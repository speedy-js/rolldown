use std::collections::HashMap;
use std::env;
use std::io;
use std::mem;
use std::path::{Path, MAIN_SEPARATOR};

use ahash::RandomState;
use once_cell::sync::Lazy;
use swc_common::sync::RwLock;

static CURRENT_DIR: Lazy<String> =
  Lazy::new(|| env::current_dir().unwrap().to_str().unwrap().to_owned());

fn resolve(path: &str) -> String {
  let mut result = String::with_capacity(CURRENT_DIR.len() + path.len() + 1);
  result.push_str(CURRENT_DIR.as_str());
  result.push(MAIN_SEPARATOR);
  result.push_str(path);
  result
}

#[derive(Clone)]
#[non_exhaustive]
pub struct HookDriver;

impl Default for HookDriver {
  fn default() -> Self {
    Self::new()
  }
}

impl HookDriver {
  pub fn new() -> Self {
    HookDriver
  }

  // build hooks
  pub fn options() {}

  pub fn build_start() {}

  pub fn resolve_id(
    &self,
    source: &str,
    importer: Option<&str>,
    parent_dir_cache: &RwLock<HashMap<String, String, RandomState>>,
  ) -> Option<String> {
    if Path::new(source).is_absolute() {
      return Some(source.to_owned());
    };

    if importer.is_none() {
      return Some(resolve(source));
    }

    if !source.starts_with('.') {
      // TODO: resolve external module
      // ignore all external module for now
      return None;
    }
    let importer = importer?;
    let read_lock = parent_dir_cache.read();
    if let Some(parent) = read_lock.get(importer) {
      let importer_dir = Path::new(parent);
      let mut result = importer_dir.join(source);
      mem::drop(read_lock);
      result.set_extension("js");
      result.to_str().map(|p| p.to_owned())
    } else {
      mem::drop(read_lock);
      let importer_dir = Path::new(importer).parent()?;
      let mut write_cache = parent_dir_cache.write();
      write_cache.insert(
        importer.to_owned(),
        importer_dir.to_str().unwrap().to_owned(),
      );
      mem::drop(write_cache);
      let mut result = importer_dir.join(source);
      result.set_extension("js");
      result.to_str().map(|p| p.to_owned())
    }
  }

  pub fn load(&self, id: &str) -> io::Result<String> {
    std::fs::read_to_string(id)
  }

  pub fn transform() {}

  pub fn module_parsed() {}

  pub fn resolve_dynamic_import() {}

  pub fn build_end() {}
}
