//!
use crate::tag::Tag;
use crate::task::{Task, TaskStatus};
use nanoid::nanoid;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::PathBuf;

const ALPHABET: [char; 30] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'w',
    'x', 'y', 'z', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub struct TaskRepo {
    conn: Connection,
}

pub fn get_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();
    path.push("itm");
    std::fs::create_dir_all(&path).ok();
    path.push("tasks.db");
    path
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
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
        );",
    )
}

pub fn open_db() -> rusqlite::Result<Connection> {
    let path = get_db_path();
    Connection::open(path)
}

impl TaskRepo {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    /// Adds a new task to the database and generates a unique identifier for it.
    ///
    /// # Arguments
    ///
    /// * `task` - A reference to a `Task` struct containing the details of the task
    ///   to be added, including its `name` and `status`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `String` which is the unique identifier
    /// (ID) of the newly added task if the operation is successful. If an error
    /// occurs during the database operation, it returns a `rusqlite::Error`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the database operation fails to insert
    /// the new task into the `tasks` table. Reasons for failure could include, but
    /// are not limited to:
    /// - Database connection issues.
    /// - Violation of any table constraints (e.g., unique constraints).
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task {
    ///     name: "Write documentation".to_string(),
    ///     status: "Pending".to_string(),
    /// };
    /// let id = tasks_repo.add(&task)?;
    /// println!("Task added with ID: {}", id);
    /// ```
    ///
    /// # Dependencies
    ///
    /// - Uses the `nanoid` crate to generate a unique 4-character task ID
    ///   using a custom `ALPHABET`.
    /// - Relies on the `rusqlite` crate for database operations.
    pub fn add(&self, task: &Task) -> rusqlite::Result<String> {
        let id = nanoid!(4, &ALPHABET);
        self.conn.execute(
            "INSERT INTO tasks (id, name, status) VALUES (?1, ?2, ?3)",
            params![id, task.name, task.status],
        )?;
        Ok(id)
    }

    /// Retrieves a `Task` by its `task_id` from the database, along with its associated tags.
    ///
    /// # Arguments
    ///
    /// * `task_id` - A `String` representing the unique identifier of the task to be fetched.
    ///
    /// # Returns
    ///
    /// Returns a `rusqlite::Result` which, on success, contains:
    /// - `Some(Task)` if the task is found in the database, with its associated `Tag` objects populated.
    /// - `None` if no task with the given `task_id` exists in the database.
    ///
    /// # Errors
    ///
    /// This function may return a `rusqlite::Error` in the following scenarios:
    /// - If the SQL query fails to prepare or execute.
    /// - If there is an error while mapping rows to the `Task` or `Tag` structs.
    /// - If there are issues with database connections or operations.
    ///
    /// # Behavior
    ///
    /// 1. Queries the `tasks` table to fetch the task details (`id`, `name`, `status`) matching the given `task_id`.
    /// 2. If a matching task is found:
    ///    - Queries the `tags` and `task_tags` join table to fetch all associated tags for the task.
    ///    - Populates the `tags` into the task using the `add_tag` method.
    /// 3. Returns the complete `Task` populated with its tags, wrapped in `Some`.
    /// 4. If no matching task is found, returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let task_id = "123".to_string();
    /// match task_db.get(task_id) {
    ///     Ok(Some(task)) => {
    ///         println!("Task found: {:?}", task);
    ///     }
    ///     Ok(None) => {
    ///         println!("Task not found.");
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error retrieving task: {}", err);
    ///     }
    /// }
    /// ```
    pub fn get(&self, task_id: &str) -> rusqlite::Result<Option<Task>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, status FROM tasks WHERE id = ?1")?;

        let task = stmt
            .query_one(params![task_id], |row| {
                Ok(Task {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    status: row.get(2)?,
                    tags: vec![],
                })
            })
            .optional()?;

        match task {
            None => Ok(None),
            Some(mut t) => {
                stmt = self.conn.prepare(
                    "
            SELECT tags.name, tags.color
                FROM tags INNER JOIN task_tags ON tags.name = task_tags.tag
            WHERE task_id = ?1",
                )?;

                let tags = stmt
                    .query_map(params![task_id], |row| {
                        Ok(Tag {
                            name: row.get(0)?,
                            color: row.get(1)?,
                        })
                    })?
                    .collect::<rusqlite::Result<Vec<Tag>>>()?;

                for tag in tags {
                    t.add_tag(tag);
                }
                Ok(Some(t))
            }
        }
    }

    pub fn list(&self) -> rusqlite::Result<Vec<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, name, status FROM tasks")?;

        let mut tasks = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    status: row.get(2)?,
                    tags: vec![],
                })
            })?
            .collect::<rusqlite::Result<Vec<Task>>>()?;

        for task in tasks.iter_mut() {
            stmt = self.conn.prepare(
                "
                SELECT tags.name, tags.color
                    FROM tags INNER JOIN task_tags ON tags.name = task_tags.tag
                WHERE task_id = ?1",
            )?;

            let tags = stmt
                .query_map(params![task.id], |row| {
                    Ok(Tag {
                        name: row.get(0)?,
                        color: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<Tag>>>()?;

            for tag in tags {
                task.add_tag(tag);
            }
        }

        Ok(tasks)
    }

    pub fn delete(&self, task_id: String) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", (task_id,))?;
        Ok(())
    }

    pub fn set_status(&self, task_id: String, status: TaskStatus) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            (status, task_id),
        )?;
        Ok(())
    }

    pub fn add_tag(&self, task_id: &String, tag: &Tag) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (name, color) VALUES (?1, ?2)",
            params![tag.name, tag.color],
        )?;

        self.conn.execute(
            "INSERT OR IGNORE INTO task_tags (task_id, tag) VALUES (?1, ?2)",
            params![task_id, tag.name],
        )?;

        Ok(())
    }

    pub fn remove_tag(&self, task_id: &String, tag: &String) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1 AND tag = ?2",
            params![task_id, tag],
        )?;
        Ok(())
    }
}
