use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::path::PathBuf;

use crate::fn_args;
use crate::{FuncSpace, SpaceKind};

pub trait MetricGroup {
    fn get_name(&self) -> &'static str;
}

impl MetricGroup for fn_args::Stats {
    fn get_name(&self) -> &'static str {
        "nargs"
    }
}

impl Serialize for Box<dyn MetricGroup> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("metrics", 1)?;
        st.serialize_field(self.get_name(), &self)?;
        st.end()
    }
}

#[derive(Serialize)]
struct SpaceData {
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: SpaceKind,
    pub spaces: Vec<FuncSpace>,
    pub metrics: Vec<Box<dyn MetricGroup>>,
}

impl SpaceData {
    pub fn new(space: &FuncSpace) -> Self {
        SpaceData {
            name: space.name.clone(),
            start_line: space.start_line,
            end_line: space.end_line,
            kind: space.kind,
            spaces: space.spaces.clone(),
            metrics: vec![Box::new(space.metrics.nargs.clone())],
        }
    }
}

//fn generate_html_files(output_dir: &PathBuf) {}

pub(crate) fn write(
    input_path: &PathBuf,
    output_path: &PathBuf,
    space: &FuncSpace,
) -> std::io::Result<()> {
    Ok(())
}
