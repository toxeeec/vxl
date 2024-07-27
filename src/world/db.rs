use std::{str::FromStr, sync::Arc};

use bevy::{prelude::*, tasks::block_on};
use sqlx::{
    migrate::MigrateDatabase,
    prelude::FromRow,
    sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow, SqliteSynchronous,
    },
    Connection, QueryBuilder, Row, Sqlite, SqliteConnection, SqlitePool,
};

use super::{Chunk, CHUNK_VOLUME};

#[derive(Resource, Clone, Debug)]
pub(super) struct Db(pub(super) SqlitePool);

#[derive(sqlx::FromRow, Debug)]
pub(super) struct ChunkRow {
    pub(super) x: i32,
    pub(super) z: i32,
    #[sqlx(flatten)]
    pub(super) blocks: Chunk,
}

impl Db {
    pub(super) async fn insert_chunks<I>(&self, chunks: I)
    where
        I: IntoIterator<Item = (IVec2, Arc<Chunk>)>,
    {
        let mut query_builder: QueryBuilder<Sqlite> =
            sqlx::QueryBuilder::new("insert into chunks (x, z, blocks) ");
        query_builder.push_values(chunks, |mut b, (offset, chunk)| {
            let blocks: Vec<_> = chunk.0.iter().map(|&block| block as u8).collect();
            b.push_bind(offset.x).push_bind(offset.y).push_bind(blocks);
        });
        let query = query_builder.build();

        query.execute(&self.0).await.unwrap();
    }

    pub(super) async fn get_chunks<'a, I>(&self, offsets: I) -> Vec<ChunkRow>
    where
        I: IntoIterator<Item = &'a IVec2>,
    {
        let mut query_builder: QueryBuilder<Sqlite> =
            sqlx::QueryBuilder::new("select x, z, blocks from chunks where (x, z) in");
        query_builder.push_tuples(offsets, |mut b, offset| {
            b.push_bind(offset.x).push_bind(offset.y);
        });
        let query = query_builder.build_query_as();
        query.fetch_all(&self.0).await.unwrap()
    }
}

impl Default for Db {
    fn default() -> Self {
        Self(block_on(async {
            let schema = sqlx::query!("select sql from sqlite_master where type='table'")
                .fetch_all(
                    &mut SqliteConnection::connect(env!("DATABASE_URL"))
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();

            Sqlite::create_database(":memory:").await.unwrap();
            let pool = SqlitePoolOptions::new()
                .max_connections(4)
                .connect_with(
                    SqliteConnectOptions::from_str(":memory:")
                        .unwrap()
                        .journal_mode(SqliteJournalMode::Wal)
                        .synchronous(SqliteSynchronous::Normal),
                )
                .await
                .unwrap();

            for statement in schema {
                if let Some(sql) = statement.sql {
                    sqlx::query(&sql).execute(&pool).await.unwrap();
                }
            }

            pool
        }))
    }
}

impl FromRow<'_, SqliteRow> for Chunk {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let blocks: [u8; CHUNK_VOLUME] = row.try_get::<&[u8], _>("blocks")?.try_into().unwrap();
        Ok(Self(blocks.map(|block| block.into())))
    }
}
