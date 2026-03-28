use sqlx::PgPool;

/// Get confirmation code count from the database
pub async fn get_confirmation_code_count<'a>(db_conn: &PgPool) -> i64 {
    let (count,) = sqlx::query_as("SELECT COUNT(*) FROM confirmation_codes;")
        .fetch_one(db_conn)
        .await
        .unwrap();
    count
}

/// Get confirmation code count from the database
pub async fn get_update_data_count<'a>(db_conn: &PgPool) -> i64 {
    let (count,) = sqlx::query_as("SELECT COUNT(*) FROM updates_data;")
        .fetch_one(db_conn)
        .await
        .unwrap();
    count
}