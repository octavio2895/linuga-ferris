#[derive(Debug)]
pub struct VocabEntry {
    pub lemma: String,
    pub pos: String,
    pub translation: Option<String>,
    pub source_form: String,
    pub context: Option<String>,
}

pub struct VocabDb {
    conn: rusqlite::Connection,
}

impl VocabDb {
    pub fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            "
    CREATE TABLE IF NOT EXISTS vocab (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        lemma       TEXT NOT NULL,
        pos         TEXT NOT NULL,
        translation TEXT,
        source_form TEXT NOT NULL,
        context     TEXT,
        added_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(lemma, pos)
    );
",
        )?;
        Ok(Self { conn })
    }

    pub fn save(&self, entry: &VocabEntry) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO vocab (lemma, pos, translation, source_form, context)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                entry.lemma,
                entry.pos,
                entry.translation,
                entry.source_form,
                entry.context
            ],
        )?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<VocabEntry>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, pos, translation, source_form, context FROM vocab")?;
        let entries = stmt
            .query_map([], |row| {
                Ok(VocabEntry {
                    lemma: row.get(0)?,
                    pos: row.get(1)?,
                    translation: row.get(2)?,
                    source_form: row.get(3)?,
                    context: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }
}
