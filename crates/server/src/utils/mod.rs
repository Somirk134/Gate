use uuid::Uuid;

pub fn generate_id() -> Uuid {
    Uuid::new_v4()
}

pub fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}
