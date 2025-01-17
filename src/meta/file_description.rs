use crate::impl_param_described;
use crate::params::{Param, ParamDescribed, ParamList};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceFile {
    pub name: String,
    pub location: String,
    pub id: String,
    pub file_format: Option<Param>,
    pub id_format: Option<Param>,
    pub params: ParamList,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileDescription {
    pub contents: ParamList,
    pub source_files: Vec<SourceFile>,
}

impl_param_described!(SourceFile);

impl ParamDescribed for FileDescription {
    fn params(&self) -> &[Param] {
        &self.contents
    }

    fn params_mut(&mut self) -> &mut ParamList {
        &mut self.contents
    }
}
