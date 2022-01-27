use std::sync::Arc;

use once_cell::sync::Lazy;
use swc::Compiler;
use swc_common::{sync::Lrc, FilePathMapping, SourceMap};

pub(crate) static SOURCE_MAP: Lazy<Lrc<SourceMap>> = Lazy::new(Default::default);

pub(crate) static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  Arc::new(Compiler::new(cm))
});
