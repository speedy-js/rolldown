use crate::{module::Module, external_module::ExternalModule};


#[derive(Debug)]
pub struct  OutputChunk {
  pub code: String,
  pub file_name: String,
}

#[derive(Debug)]
pub struct RenderedChunk {
  pub code: String,
  pub file_name: String,
}


#[derive(Debug)]
pub enum RolldownOutput {
  Chunk(OutputChunk),
  Asset,
}

impl RolldownOutput {
  #[inline]
  pub fn get_file_name(&self) -> &str {
    match self {
      RolldownOutput::Chunk(c) => c.file_name.as_ref(),
      RolldownOutput::Asset => panic!(""),
    }
  }

  #[inline]
  pub fn get_content(&self) -> &str {
    match self {
      RolldownOutput::Chunk(c) => c.code.as_ref(),
      RolldownOutput::Asset => panic!(""),
    }
  }
}


#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ModOrExt {
  Mod(Module),
  Ext(ExternalModule),
}