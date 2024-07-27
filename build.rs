use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = "sqlite://schema.db";
    if Sqlite::database_exists(db_url).await? {
        Sqlite::drop_database(db_url).await?;
    }
    Sqlite::create_database(db_url).await?;

    let mut conn = SqliteConnection::connect(db_url).await?;
    sqlx::query(
        "create table chunks 
         (x integer not null, z integer not null, blocks blob not null, primary key (x, z)) 
         strict, without rowid",
    )
    .execute(&mut conn)
    .await?;

    println!("cargo:rustc-env=DATABASE_URL={}", db_url);
    Ok(())
}
