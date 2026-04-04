use sqlx::{Acquire, Postgres};
use tracing::instrument;
use crate::{command::permissions::{create::create_permission, delete::delete_permission}, error::command::permission::PermissionSyncError, settings::permissions::PermissionSettings, utils::vec::detect_differences_in_vecs};

/// Command to sync permissions from the config to the database.
#[instrument(name = "Syncing permissions with the config.", skip(db_conn, permission_settings))]
pub async fn sync_permissions<'a, A: Acquire<'a, Database = Postgres>>(
    db_conn: A,
    permission_settings: &PermissionSettings
) -> Result<(), PermissionSyncError> {
    let mut db_conn = db_conn.acquire().await?;

    // get all permissions
    let sql = "SELECT name FROM permissions;";
    let in_database = {
        let db_result: Vec<(String,)> = sqlx::query_as(sql)
            .fetch_all(&mut *db_conn)
            .await?;
        db_result
            .iter()
            .map(|row: &(String,)| row.0.clone())
            .collect::<Vec<String>>()
    };

    let in_config = permission_settings
        .as_ref()
        .clone();

    let result = detect_differences_in_vecs(in_database, in_config);

    if result.is_none() {
        // no changes need to be made
        return Ok(());
    }

    let result = result.unwrap();

    for permission_name in result.create {
        let _ = create_permission(&mut *db_conn, &permission_name).await;
    }

    for permission_name in result.delete {
        let _ = delete_permission(&mut *db_conn, &permission_name).await;
    }

    Ok(())
}