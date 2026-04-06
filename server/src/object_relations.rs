/// This file contains the ORM for the server, it sits as an abstraction layer between SQLite and custom data types
use rusqlite::{Connection, Result, params};
use super::blocks::{Block, Chest};
use super::turtle::Turtle;

pub struct ORM {
    conn: Connection,
}

impl ORM {
    pub fn new(url: Option<String>) -> Self {
        let conn = match url {
            Some(path) => Connection::open(path).expect("Failed to open database"),
            None => Connection::open_in_memory().expect("Failed to open in-memory database"),
        };

        ORM { conn }
    }

    pub fn create_tables(&self) -> Result<()> {
        // Create the blocks table with a composite primary key on (x, y, z)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                z INTEGER NOT NULL,
                block_type TEXT NOT NULL,
                PRIMARY KEY (x, y, z)
            )",
        ())?;

        // Create the chests table with a composite primary key on (x, y, z)
        self.conn.execute(
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
    }

    pub fn upsert_block(&self, block: &Block) -> Result<()> {
        self.conn.execute(
            "INSERT INTO blocks (x, y, z, block_type) VALUES (?1, ?2, ?3, ?4) ON CONFLICT (x, y, z) DO UPDATE SET block_type = excluded.block_type",
            params![block.x, block.y, block.z, block.block_type],
        )?;
        Ok(())
    }

    pub fn remove_block(&self, x: i32, y: i32, z: i32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM blocks WHERE x = ?1 AND y = ?2 AND z = ?3",
            params![x, y, z],
        )?;
        Ok(())
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        let data = self.conn.query_row(
            "SELECT block_type FROM blocks WHERE x = ?1 AND y = ?2 AND z = ?3",
            params![x, y, z],
            |row| row.get(0),
        );

        match data {
            Ok(block_type) => Some(Block { x, y, z, block_type }),
            Err(_) => None,
        }
    }

    pub fn get_all_blocks(&self) -> Result<Vec<Block>> {
        let mut stmt = self.conn.prepare("SELECT x, y, z, block_type FROM blocks")?;

        let block_iter = stmt.query_map([], |row| {
            Ok(Block {
                x: row.get(0)?,
                y: row.get(1)?,
                z: row.get(2)?,
                block_type: row.get(3)?,
            })
        })?;

        let mut blocks = Vec::new();

        for block in block_iter {
            blocks.push(block?);
        }

        Ok(blocks)
    }

    pub fn upsert_chest(&self, chest: &Chest) -> Result<()> {
        self.conn.execute(
            "INSERT INTO chests (x, y, z, count, max_count, item_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT (x, y, z) DO UPDATE SET count = excluded.count, max_count = excluded.max_count, item_type = excluded.item_type",
            params![chest.x, chest.y, chest.z, chest.count, chest.max_count, chest.item_type],
        )?;
        Ok(())
    }

    pub fn remove_chest(&self, x: i32, y: i32, z: i32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM chests WHERE x = ?1 AND y = ?2 AND z = ?3",
            params![x, y, z],
        )?;
        Ok(())
    }

    pub fn get_chest(&self, x: i32, y: i32, z: i32) -> Option<Chest> {
        let data = self.conn.query_row(
            "SELECT count, max_count, item_type FROM chests WHERE x = ?1 AND y = ?2 AND z = ?3",
            params![x, y, z],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match data {
            Ok((count, max_count, item_type)) => Some(Chest { x, y, z, count, max_count, item_type }),
            Err(_) => None,
        }
    }

    pub fn get_all_chests(&self) -> Result<Vec<Chest>> {
        let mut stmt = self.conn.prepare("SELECT x, y, z, count, max_count, item_type FROM chests")?;

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
    }

    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM blocks", [])?;
        self.conn.execute("DELETE FROM chests", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_crud() {
        let orm = ORM::new(None);
        orm.create_tables().unwrap();

        let block = Block { x: 1, y: 2, z: 3, block_type: "stone".to_string() };
        orm.upsert_block(&block).unwrap();

        let retrieved = orm.get_block(1, 2, 3).unwrap();
        assert_eq!(retrieved.block_type, "stone");

        orm.remove_block(1, 2, 3).unwrap();
        assert!(orm.get_block(1, 2, 3).is_none());

        let block_count = orm.get_all_blocks().unwrap().len();
        assert_eq!(block_count, 0);
    }

    #[test]
    fn test_chest_crud() {
        let orm = ORM::new(None);
        orm.create_tables().unwrap();

        let chest = Chest { x: 4, y: 5, z: 6, count: 10, max_count: 20, item_type: "diamond".to_string() };
        orm.upsert_chest(&chest).unwrap();

        let retrieved = orm.get_chest(4, 5, 6).unwrap();
        assert_eq!(retrieved.item_type, "diamond");
        assert_eq!(retrieved.count, 10);
        assert_eq!(retrieved.max_count, 20);

        orm.remove_chest(4, 5, 6).unwrap();
        assert!(orm.get_chest(4, 5, 6).is_none());

        let chest_count = orm.get_all_chests().unwrap().len();
        assert_eq!(chest_count, 0);
    }

    #[test]
    fn test_clear() {
        let orm = ORM::new(None);
        let block = Block { x: 1, y: 2, z: 3, block_type: "stone".to_string() };
        let chest = Chest { x: 4, y: 5, z: 6, count: 10, max_count: 20, item_type: "diamond".to_string() };
        orm.create_tables().unwrap();
        orm.upsert_block(&block).unwrap();
        orm.upsert_chest(&chest).unwrap();
        orm.clear().unwrap();
        assert!(orm.get_block(1, 2, 3).is_none());
        assert!(orm.get_chest(4, 5, 6).is_none());
    }
}