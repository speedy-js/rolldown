use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{ffi::OsString, path::Path};

use crate::ext::StrExt;
use crate::{
  ext::PathExt, plugin_driver::PluginDriver, types::ResolvedId, utils::is_external_module,
};

// from require("module").builtinModules
static BUILTIN_MODULES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
  HashSet::from([
    "_http_agent",
    "_http_client",
    "_http_common",
    "_http_incoming",
    "_http_outgoing",
    "_http_server",
    "_stream_duplex",
    "_stream_passthrough",
    "_stream_readable",
    "_stream_transform",
    "_stream_wrap",
    "_stream_writable",
    "_tls_common",
    "_tls_wrap",
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
  ])
});

pub fn resolve_id(
  source: &str,
  importer: Option<&str>,
  preserve_symlinks: bool,
  plugin_driver: &PluginDriver,
) -> ResolvedId {
  let plugin_result = resolve_id_via_plugins(source, importer, plugin_driver);

  plugin_result.unwrap_or_else(|| {
    let res = if importer.is_some() && is_external_module(source) {
      let normalized_source = source.replace("node:", "");
      if BUILTIN_MODULES.contains(normalized_source.as_str()) {
        ResolvedId::new(normalized_source, true)
      } else {
        let raw_id =
          node_resolve::resolve_from(normalized_source.as_str(), importer.unwrap().as_path());
        log::debug!("resolving external module {:#?}", normalized_source);
        match raw_id {
          Ok(id) => {
            let file: &Path = id.as_ref();
            // External should be judged based on `external options`
            ResolvedId::new(file.to_string_lossy().to_string(), false)
          }
          Err(_) => panic!("Module {} is not exist.", normalized_source),
        }
      }
    } else {
      let id = if let Some(importer) = importer {
        nodejs_path::resolve!(&nodejs_path::dirname(importer), source)
      } else {
        nodejs_path::resolve!(source)
      };
      ResolvedId::new(
        fast_add_js_extension_if_necessary(id, preserve_symlinks),
        false,
      )
    };
    res
  })
}

fn resolve_id_via_plugins(
  source: &str,
  importer: Option<&str>,
  plugin_driver: &PluginDriver,
) -> Option<ResolvedId> {
  plugin_driver.resolve_id(source, importer)
}

#[inline]
fn fast_add_js_extension_if_necessary(mut file: String, _preserve_symlinks: bool) -> String {
  if !file.ends_with(".js") {
    file.push_str(".js");
  }
  file
}

fn add_js_extension_if_necessary(file: &str, preserve_symlinks: bool) -> String {
  let found = find_file(Path::new(file), preserve_symlinks);
  found.unwrap_or_else(|| {
    let found = find_file(Path::new(&(file.to_string() + "#.mjs")), preserve_symlinks);
    found.unwrap_or_else(|| {
      let found = find_file(Path::new(&(file.to_string() + ".js")), preserve_symlinks);
      found.unwrap()
    })
  })
}

fn find_file(file: &Path, preserve_symlinks: bool) -> Option<String> {
  let metadata = std::fs::metadata(file);
  if let Ok(metadata) = metadata {
    if !preserve_symlinks && metadata.is_symlink() {
      find_file(&std::fs::canonicalize(file).unwrap(), preserve_symlinks)
    } else if (preserve_symlinks && metadata.is_symlink()) || metadata.is_file() {
      let name: OsString = nodejs_path::basename!(&file.as_str()).into();
      let files = std::fs::read_dir(&nodejs_path::dirname(&file.as_str())).unwrap();
      let s = files
        .map(|result| result.unwrap())
        .find(|file| file.file_name() == name)
        .map(|_| file.to_string_lossy().to_string());
      s
    } else {
      None
    }
  } else {
    None
  }
}
