use std::path::PathBuf;
use rusqlite::{params, Connection, OptionalExtension};
use crate::task::{TaskStatus, Task};
use nanoid::nanoid;

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
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            status INTEGER NOT NULL
        )"
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
                status: row.get(2)?
            })
        }).optional();
        task
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

        let tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                status: row.get(2)?
            })
        })?.collect::<rusqlite::Result<Vec<Task>>>()?;

        Ok(tasks)

    }

    pub fn delete(&self, task_id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", (task_id,))?;
        Ok(())
    }

}