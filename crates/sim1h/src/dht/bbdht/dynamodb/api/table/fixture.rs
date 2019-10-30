use uuid::Uuid;

pub fn table_name_fresh() -> String {
    format!("table_{}", Uuid::new_v4())
}
