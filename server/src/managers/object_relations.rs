use std::env;

/// This file contains the ORM for the server, it sits as an abstraction layer between SQLite and custom data types
use tokio_rusqlite::{Connection, Result, params};
use shared::blocks::{Block, Chest};

pub struct ORM {
    conn: Connection,
}

impl ORM {
    pub async fn new() -> Self {
        let url = match env::var("DATABASE_URL") {
            Ok(val) => {
                println!("Using database at path: {}", val);
                Some(val)
            },
            Err(_) => {
                println!("No database path provided, using in-memory database.");
                None
            },
        };
        
        let conn = match url {
            Some(path) => Connection::open(path).await.expect("Failed to open database"),
            None => Connection::open("file:memdb1?mode=memory&cache=shared").await.expect("Failed to open in-memory database"),
        };

        conn.call(|conn| -> Result<()> {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "busy_timeout", "5000")?;
            Ok(())
        }).await.unwrap();

        let orm = ORM { conn };
        orm.create_tables().await.expect("Failed to create database tables");
        orm
    }

    pub async fn create_tables(&self) -> Result<()> {
        // Create the blocks table with a composite primary key on (x, y, z)
        self.conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS blocks (
                    x INTEGER NOT NULL,
                    y INTEGER NOT NULL,
                    z INTEGER NOT NULL,
                    block_type TEXT NOT NULL,
                    last_updated INTEGER DEFAULT (strftime('%s', 'now')),
                    PRIMARY KEY (x, y, z)
                )",
            ())?;
    
            // Create the chests table with a composite primary key on (x, y, z)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS chests (
                    x INTEGER NOT NULL,
                    y INTEGER NOT NULL,
                    z INTEGER NOT NULL,
                    count INTEGER NOT NULL,
                    max_count INTEGER NOT NULL,
                    item_type TEXT NOT NULL,
                    PRIMARY KEY (x, y, z)
                )",
            ())?;

            Ok(())
        }).await?;

        Ok(())
    }

    pub async fn upsert_block(&self, block: Block) -> Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO blocks (x, y, z, block_type, last_updated) VALUES (?1, ?2, ?3, ?4, strftime('%s', 'now')) ON CONFLICT (x, y, z) DO UPDATE SET block_type = excluded.block_type, last_updated = excluded.last_updated",
                params![block.x, block.y, block.z, block.block_type],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn remove_block(&self, x: i64, y: i64, z: i64) -> Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "DELETE FROM blocks WHERE x = ?1 AND y = ?2 AND z = ?3",
                params![x, y, z],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn get_block(&self, x: i64, y: i64, z: i64) -> Option<Block> {
        let data = self.conn.call(move |conn| {
            conn.query_row(
                "SELECT block_type, last_updated FROM blocks WHERE x = ?1 AND y = ?2 AND z = ?3",
                params![x, y, z],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
        }).await.ok();
        
        match data {
            Some((block_type, last_updated)) => Some(Block { x, y, z, block_type, last_updated }),
            None => None,
        }
    }

    pub async fn get_all_blocks(&self) -> Result<Vec<Block>> {
        let blocks = self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT x, y, z, block_type, last_updated FROM blocks")?;

            let block_iter = stmt.query_map([], |row| {
                Ok(Block {
                    x: row.get(0)?,
                    y: row.get(1)?,
                    z: row.get(2)?,
                    block_type: row.get(3)?,
                    last_updated: row.get(4)?,
                })
            })?;

            let mut blocks = Vec::new();
            for block in block_iter {
                blocks.push(block?);
            }
            Ok(blocks)
        }).await?;

        Ok(blocks)
    }

    pub async fn upsert_chest(&self, chest: &Chest) -> Result<()> {
        let chest = chest.clone();
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO chests (x, y, z, count, max_count, item_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT (x, y, z) DO UPDATE SET count = excluded.count, max_count = excluded.max_count, item_type = excluded.item_type",
                params![chest.x, chest.y, chest.z, chest.count, chest.max_count, chest.item_type],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn remove_chest(&self, x: i64, y: i64, z: i64) -> Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "DELETE FROM chests WHERE x = ?1 AND y = ?2 AND z = ?3",
                params![x, y, z],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn get_chest(&self, x: i64, y: i64, z: i64) -> Option<Chest> {
        let data = self.conn.call(move |conn| {
            conn.query_row(
                "SELECT count, max_count, item_type FROM chests WHERE x = ?1 AND y = ?2 AND z = ?3",
                params![x, y, z],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
        }).await.ok();

        match data {
            Some((count, max_count, item_type)) => Some(Chest { x, y, z, count, max_count, item_type }),
            None => None,
        }
    }

    pub async fn get_all_chests(&self) -> Result<Vec<Chest>> {
        let chests = self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT x, y, z, count, max_count, item_type FROM chests")?;

            let chest_iter = stmt.query_map([], |row| {
                Ok(Chest {
                    x: row.get(0)?,
                    y: row.get(1)?,
                    z: row.get(2)?,
                    count: row.get(3)?,
                    max_count: row.get(4)?,
                    item_type: row.get(5)?,
                })
            })?;

            let mut chests = Vec::new();
            for chest in chest_iter {
                chests.push(chest?);
            }
            Ok(chests)
        }).await?;

        Ok(chests)
    }

    pub async fn clear(&self) -> Result<()> {
        self.conn.call(|conn| {
            conn.execute("DELETE FROM blocks", [])?;
            conn.execute("DELETE FROM chests", [])?;
            Ok(())
        }).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_block_crud() {
        unsafe {
            env::set_var("DATABASE_URL", "file:testdb_1?mode=memory&cache=shared");
        }

        let orm = ORM::new().await;
        orm.create_tables().await.unwrap();

        let block = Block { x: 1, y: 2, z: 3, block_type: "stone".to_string(), last_updated: 0 };
        orm.upsert_block(block).await.unwrap();

        let retrieved = orm.get_block(1, 2, 3).await.unwrap();
        assert_eq!(retrieved.block_type, "stone");
        assert_ne!(retrieved.last_updated, 0);

        orm.remove_block(1, 2, 3).await.unwrap();
        assert!(orm.get_block(1, 2, 3).await.is_none());

        let block_count = orm.get_all_blocks().await.unwrap().len();
        assert_eq!(block_count, 0);
    }

    #[tokio::test]
    async fn test_chest_crud() {
        unsafe {
            env::set_var("DATABASE_URL", "file:testdb_2?mode=memory&cache=shared");
        }

        let orm = ORM::new().await;
        orm.create_tables().await.unwrap();

        let chest = Chest { x: 4, y: 5, z: 6, count: 10, max_count: 20, item_type: "diamond".to_string() };
        orm.upsert_chest(&chest).await.unwrap();

        let retrieved = orm.get_chest(4, 5, 6).await.unwrap();
        assert_eq!(retrieved.item_type, "diamond");
        assert_eq!(retrieved.count, 10);
        assert_eq!(retrieved.max_count, 20);

        orm.remove_chest(4, 5, 6).await.unwrap();
        assert!(orm.get_chest(4, 5, 6).await.is_none());

        let chest_count = orm.get_all_chests().await.unwrap().len();
        assert_eq!(chest_count, 0);
    }

    #[tokio::test]
    async fn test_clear() {
        unsafe {
            env::set_var("DATABASE_URL", "file:testdb_3?mode=memory&cache=shared");
        }

        let orm = ORM::new().await;
        let block = Block { x: 1, y: 2, z: 3, block_type: "stone".to_string(), last_updated: 0 };
        let chest = Chest { x: 4, y: 5, z: 6, count: 10, max_count: 20, item_type: "diamond".to_string() };
        orm.create_tables().await.unwrap();
        orm.upsert_block(block).await.unwrap();
        orm.upsert_chest(&chest).await.unwrap();
        orm.clear().await.unwrap();
        assert!(orm.get_block(1, 2, 3).await.is_none());
        assert!(orm.get_chest(4, 5, 6).await.is_none());
    }
}