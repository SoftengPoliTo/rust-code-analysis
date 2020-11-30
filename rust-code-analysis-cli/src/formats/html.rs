use halstead::Halstead;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

use crate::fn_args;
use crate::halstead;
use crate::{FuncSpace, SpaceKind};

/// A field within the metric
#[derive(Serialize, Debug, Clone)]
struct MetricField {
    /// Name of the field
    pub name: String,
    /// format!ed value. of the field
    pub value: String,
}

impl MetricField {
    fn from_f64(name: &str, value: f64) -> Self {
        Self {
            name: name.into(),
            value: format!("{:.3}", value),
        }
    }
}

/// A metric
#[derive(Serialize, Debug, Clone)]
struct Metric {
    pub name: String,
    /// All the important metric fields
    pub fields: Vec<MetricField>,
    /// The most representative
    pub summary: MetricField,
}

/// All the data about the current Space
#[derive(Serialize, Debug, Clone)]
struct SpaceData {
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: SpaceKind,
    /// SubSpaces within the current space
    pub spaces: Vec<SpaceData>,
    /// Its parent if present
    pub parent: Option<Box<SpaceData>>,
    /// List of the metrics for the current space
    pub metrics: Vec<Metric>,
}

impl SpaceData {
    fn from_parent(space: &FuncSpace, parent: Option<Box<SpaceData>>) -> SpaceData {
        let mut data = SpaceData {
            name: space.name.clone(),
            start_line: space.start_line,
            end_line: space.end_line,
            kind: space.kind,
            spaces: Vec::new(),
            parent,
            metrics: build_metrics(&space.metrics),
        };

        let spaces = space.spaces.iter().map(|s| {
            SpaceData::from_parent(s, Some(data.clone().into()))
        }).collect::<Vec<_>>();

        data.spaces = spaces;

        data
    }
}

use crate::CodeMetrics;

impl From<&fn_args::Stats> for Metric {
    fn from(stats: &fn_args::Stats) -> Metric {
        Metric {
            name: "NArgs".into(),
            fields: vec![
                MetricField::from_f64("Sum", stats.nargs()),
                MetricField::from_f64("Average", stats.nargs_average()),
            ],
            summary: MetricField::from_f64("NArgs - Average", stats.nargs()),
        }
    }
}

impl From<&halstead::Stats> for Metric {
    fn from(stats: &halstead::Stats) -> Metric {
        Metric {
            name: "Halstead".into(),
            fields: vec![
                MetricField::from_f64("Difficulty", stats.difficulty()),
                MetricField::from_f64("Effort", stats.effort()),
                MetricField::from_f64("Bugs", stats.bugs()),
            ],
            summary: MetricField::from_f64("Halstead - Difficulty", stats.difficulty()),
        }
    }
}

fn build_metrics(metrics: &CodeMetrics) -> Vec<Metric> {
    let mut out = Vec::new();
    out.push(Metric::from(&metrics.nargs));
    out.push(Metric::from(&metrics.halstead));

    out
}

impl From<&FuncSpace> for SpaceData {
    fn from(space: &FuncSpace) -> SpaceData {
        SpaceData::from_parent(space, None)
    }
}

pub(crate) fn write(
    input_path: &PathBuf,
    output_path: &PathBuf,
    space: &FuncSpace,
) -> std::io::Result<()> {

//    println!("{:?}", space);

    let s = SpaceData::from(space);
    println!("{:#?}", s);

    Ok(())
}
