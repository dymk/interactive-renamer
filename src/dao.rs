use std::path::Path;

use rusqlite::{self, params, Connection};

use crate::app_state::mapping_state::MappedDir;

pub struct Dao {
    conn: Connection,
}
impl Dao {
    pub fn new(db_path: &str) -> Dao {
        let conn = Connection::open(db_path).unwrap();
        Dao::migrate(&conn);
        Dao { conn }
    }

    fn migrate(conn: &Connection) {
        conn.execute(
            r"
            CREATE TABLE IF NOT EXISTS dir_mappings (
                in_path TEXT PRIMARY KEY,
                ext_filter TEXT,
                dir_matcher TEXT,
                dir_replacer TEXT,
                file_matcher TEXT,
                file_replacer TEXT
            ) WITHOUT ROWID;
        ",
            [],
        )
        .unwrap();
    }

    pub fn get_mapped_dir_by_in_path(&self, in_path: &str) -> Option<MappedDir> {
        let mut stmt = self
            .conn
            .prepare_cached(
                r"
        SELECT
            in_path,
            ext_filter,
            dir_matcher, 
            dir_replacer,
            file_matcher,
            file_replacer
        FROM dir_mappings
        WHERE in_path = ?
        LIMIT 1
        ",
            )
            .unwrap();

        let mut cursor = stmt.query(params![in_path]).unwrap();
        if let Some(row) = cursor.next().unwrap() {
            let cols: [String; 6] = [
                row.get(0).unwrap(),
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
                row.get(5).unwrap(),
            ];
            Some(MappedDir::deserialize(cols))
        } else {
            None
        }
    }

    pub fn upsert_mapped_dir(&self, mapped_dir: &MappedDir) {
        let mut stmt = self
            .conn
            .prepare_cached(
                r"
        INSERT OR REPLACE INTO dir_mappings (
            in_path,
            ext_filter,
            dir_matcher, 
            dir_replacer,
            file_matcher,
            file_replacer
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
            )
            .unwrap();

        stmt.execute(mapped_dir.serialize()).unwrap();
    }
}
