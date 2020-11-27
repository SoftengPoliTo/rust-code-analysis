use num_format::{CustomFormat, Grouping};

pub(crate) struct MetricPrinter {
    columns: usize,
    row: String,
    subrow: String,
    number_format: num_format::CustomFormat,
}

impl MetricPrinter {
    pub(crate) fn new(columns: Option<usize>) -> Self {
        let columns = columns.unwrap_or(79);
        let number_format = CustomFormat::builder()
            .grouping(Grouping::Standard)
            .build()
            .unwrap();

        Self {
            columns,
            row: "=".repeat(columns),
            subrow: "-".repeat(columns),
            number_format,
        }
    }

    pub(crate) fn to_string(&self) -> std::io::Result<String> {
        Ok("".to_owned())
    }
}
