use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub type DbConnection = Arc<DatabaseConnection>;

pub fn parse_array(parse: String) -> Vec<String> {
    let mut result = Vec::new();

    let split = if let Some(prefix_stripped) = parse.strip_prefix('[') {
        if let Some(suffix_stripped) = prefix_stripped.strip_suffix(']') {
            suffix_stripped.split(',')
        } else {
            return Vec::new();
        }
    } else {
        return Vec::new();
    };

    for i in split {
        let striped = i.replace('"', "").replace(" ", "");

        result.push(striped.to_string())
    }

    result
}
