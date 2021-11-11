use anyhow::Result;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum ColumnType {
    None,
    String,
    Integer,
    Float,
}

// TODO: Use Derive once its stabilized: https://github.com/rust-lang/rust/issues/86985
impl Default for ColumnType {
    fn default() -> Self {
        ColumnType::None
    }
}

impl ColumnType {
    fn update(&mut self, value: &str) -> Result<()> {
        *self = match (
            f64::from_str(value).is_ok(),
            i64::from_str(value).is_ok(),
            &self,
        ) {
            (true, true, ColumnType::None) | (true, true, ColumnType::Integer) => {
                ColumnType::Integer
            }
            (true, false, ColumnType::None)
            | (true, _, ColumnType::Float)
            | (true, false, ColumnType::Integer) => ColumnType::Float,
            (false, false, _) | (_, _, ColumnType::String) => ColumnType::String,
            (false, true, _) => unreachable!(),
        };
        Ok(())
    }
}

fn classify_table<P: AsRef<Path>>(path: P, separator: char) -> Result<HashMap<String, ColumnType>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(separator as u8)
        .from_path(path)?;

    let headers = reader.headers()?.clone();
    let mut classification = HashMap::from_iter(
        headers
            .iter()
            .map(|f| (f.to_owned(), ColumnType::default())),
    );

    for record in reader.records() {
        let result = record?;
        for (i, title) in headers.iter().enumerate() {
            let column_type = classification.get_mut(title).unwrap();
            column_type.update(&result[i])?;
        }
    }

    Ok(classification)
}

#[cfg(test)]
mod tests {
    use crate::utils::column_type::{classify_table, ColumnType};
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_classify_table() {
        let classification =
            classify_table("tests/data/sales.csv", char::from_str(",").unwrap()).unwrap();
        let expected = HashMap::from([
            (String::from("first"), ColumnType::String),
            (String::from("last"), ColumnType::String),
            (String::from("ccnumber"), ColumnType::Integer),
            (String::from("price"), ColumnType::Float),
        ]);
        assert_eq!(classification, expected);
    }
}
