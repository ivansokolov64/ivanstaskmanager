use colored::{Color, Colorize};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy)]
pub enum TagColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl From<&str> for TagColor {
    fn from(value: &str) -> Self {
        match value {
            "red" => TagColor::Red,
            "green" => TagColor::Green,
            "yellow" => TagColor::Yellow,
            "blue" => TagColor::Blue,
            "magenta" => TagColor::Magenta,
            "cyan" => TagColor::Cyan,
            "white" => TagColor::White,
            _ => TagColor::White,
        }
    }
}

impl FromSql for TagColor {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(TagColor::Red),
            1 => Ok(TagColor::Green),
            2 => Ok(TagColor::Yellow),
            3 => Ok(TagColor::Blue),
            4 => Ok(TagColor::Magenta),
            5 => Ok(TagColor::Cyan),
            6 => Ok(TagColor::White),
            _ => Ok(TagColor::White),
        }
    }
}

impl ToSql for TagColor {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let s = match self {
            TagColor::Red => 0,
            TagColor::Green => 1,
            TagColor::Yellow => 2,
            TagColor::Blue => 3,
            TagColor::Magenta => 4,
            TagColor::Cyan => 5,
            TagColor::White => 6,
        };

        Ok(ToSqlOutput::from(s))
    }
}

impl From<TagColor> for Color {
    fn from(value: TagColor) -> Self {
        match value {
            TagColor::Red => Color::Red,
            TagColor::Green => Color::Green,
            TagColor::Yellow => Color::Yellow,
            TagColor::Blue => Color::Blue,
            TagColor::Magenta => Color::BrightMagenta,
            TagColor::Cyan => Color::Cyan,
            TagColor::White => Color::White,
        }
    }
}

pub struct Tag {
    pub name: String,
    pub color: TagColor,
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.color(Color::from(self.color)))
    }
}
