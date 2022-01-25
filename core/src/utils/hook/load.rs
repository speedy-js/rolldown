use crate::plugin_driver::PluginDriver;

pub fn load(id: &str, plugin_driver: &PluginDriver) -> String {
  log::debug!("load {}", id);
  plugin_driver
    .load(id)
    .unwrap_or_else(|| std::fs::read_to_string(id).unwrap())
}
