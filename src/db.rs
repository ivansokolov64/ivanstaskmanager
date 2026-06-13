use std::path::PathBuf;
use rusqlite::{params, Connection, OptionalExtension};
use crate::task::{TaskStatus, Task};
use nanoid::nanoid;
use crate::tag::Tag;

const ALPHABET: [char; 30] = [
    'a','b','c','d','e','f','g','h','j','k',
    'm','n','p','q','r','s','t','u','w','x',
    'y','z','2','3','4','5','6','7','8','9'
];

pub struct TaskRepo {
    conn: Connection
}

pub fn get_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();
    path.push("itm");
    std::fs::create_dir_all(&path).ok();
    path.push("tasks.db");
    path
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<()>{
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            status INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tags (
            name TEXT PRIMARY KEY,
            color INTEGER NOT NULL DEFAULT 7
        );
        CREATE TABLE IF NOT EXISTS task_tags (
            task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            tag TEXT NOT NULL REFERENCES tags(name),
            PRIMARY KEY (task_id, tag)
        );"
    )
}

pub fn open_db() -> rusqlite::Result<Connection> {
    let path = get_db_path();
    Connection::open(path)
}

impl TaskRepo {

    pub fn new(conn: Connection) -> Self {
        Self {
            conn
        }
    }
    pub fn add(&self, task: &Task) -> rusqlite::Result<String> {
        let id = nanoid!(4, &ALPHABET);
        self.conn.execute(
            "INSERT INTO tasks (id, name, status) VALUES (?1, ?2, ?3)",
            params![id, task.name, task.status]
        )?;
        Ok(id)
    }

    pub fn get(&self, task_id: String) -> rusqlite::Result<Option<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, name, status FROM tasks WHERE id = ?1")?;

        let task = stmt.query_one(params![task_id], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                status: row.get(2)?,
                tags: vec![]
            })
        }).optional()?;

        match task {
            None => {
                Ok(None)
            }
            Some(mut t) => {
                stmt = self.conn.prepare("
            SELECT tags.name, tags.color
                FROM tags INNER JOIN task_tags ON tags.name = task_tags.tag
            WHERE task_id = ?1")?;

                let tags = stmt.query_map(params![task_id], |row| {
                    Ok(Tag {
                        name: row.get(0)?,
                        color: row.get(1)?,
                    })
                })?.collect::<rusqlite::Result<Vec<Tag>>>()?;


                for tag in tags {
                    t.add_tag(tag);
                }
                Ok(Some(t))
            }
        }

    }

    pub fn set_status(&self, task_id: String, status: TaskStatus) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            (status, task_id)
        )?;
        Ok(())
    }

    pub fn list(&self) -> rusqlite::Result<Vec<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, name, status FROM tasks")?;

        let mut tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                status: row.get(2)?,
                tags: vec![]
            })
        })?.collect::<rusqlite::Result<Vec<Task>>>()?;

        for task in tasks.iter_mut() {
            stmt = self.conn.prepare("
                SELECT tags.name, tags.color
                    FROM tags INNER JOIN task_tags ON tags.name = task_tags.tag
                WHERE task_id = ?1")?;

            let tags = stmt.query_map(params![task.id], |row| {
                Ok(Tag {
                    name: row.get(0)?,
                    color: row.get(1)?,
                })
            })?.collect::<rusqlite::Result<Vec<Tag>>>()?;


            for tag in tags {
                task.add_tag(tag);
            }
        }

        Ok(tasks)

    }

    pub fn delete(&self, task_id: String) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", (task_id,))?;
        Ok(())
    }

}