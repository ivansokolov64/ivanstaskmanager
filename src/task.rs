use std::fmt::{Display, Formatter};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use colored::Colorize;
use crate::tag::Tag;

#[derive(Debug)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done
}

impl ToSql for TaskStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let s = match self {
            TaskStatus::Todo => 0,
            TaskStatus::Doing => 1,
            TaskStatus::Done => 2
        };
        Ok(ToSqlOutput::from(s))
    }
}

impl FromSql for TaskStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(TaskStatus::Todo),
            1 => Ok(TaskStatus::Doing),
            2 => Ok(TaskStatus::Done),
            other => Err(FromSqlError::Other(
                format!("Invalid status: {}", other).into()
            ))
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => {
                write!(f, "{}", "[ ]".red())
            }
            TaskStatus::Doing => {
                write!(f, "{}", "[~]".blue())
            }
            TaskStatus::Done => {
                write!(f, "{}", "[✓]".green())
            }
        }
    }
}

pub struct Task {
    pub id: Option<String>,
    pub name: String,
    pub status: TaskStatus,
    pub tags: Vec<Tag>
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id = self.id.clone().map_or("-".to_string(), |id| id.to_string());
        let tags = self.tags.iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{} {} ({}) [{}]", self.status, self.name, id, tags)
    }
}

impl Task {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            status: TaskStatus::Todo,
            tags: vec![]
        }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }
}