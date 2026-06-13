use anyhow::{Context, Result};
use crate::db::{open_db, TaskRepo};
use clap::{Parser, Subcommand};
use crate::tag::{Tag, TagColor};
use crate::task::{Task, TaskStatus};

pub mod task;
pub mod db;
pub mod tag;

// Set up the clap struct for our CLI
#[derive(Parser)]
#[command(name="Ivan's Task Manager", about="A simple CLI task manager with no fuss.")]
#[command(subcommand_required = false)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>
}

// Define the commands that can be called from the CLI
#[derive(Subcommand)]
enum Command {
    Add {
        name: String
    },
    List,
    Todo {
        id: String
    },
    Doing {
        id: String
    },
    Done {
        id: String
    },
    Delete {
        id: String
    },
    Tag {
        task_id: String,
        #[arg(num_args=1..,  allow_hyphen_values = true)]
        changes: Vec<String>
    }
}


fn print_task_list(task_list: &TaskRepo) -> Result<()> {
    let tasks = task_list.list()
        .context("Failed to get task list")?;

    println!("ALL TASKS ({})", tasks.len());
    for task in tasks {
        println!("{task}");
    }

    Ok(())
}

struct TagChanges {
    add: Vec<Tag>,
    remove: Vec<String>,
}
fn parse_tag_changes(changes: &[String]) -> Result<TagChanges> {
    let mut add = vec![];
    let mut remove = vec![];

    for change in changes {
        if let Some(rest) = change.strip_prefix('+') {
            let (name, color) = match rest.split_once(':') {
                Some((name, color_str)) => (name.to_string(), TagColor::from(color_str)),
                None                    => (rest.to_string(), TagColor::White),
            };

            add.push(Tag { name, color });

        } else if let Some(tag) = change.strip_prefix('-') {
            remove.push(tag.to_string());
        } else {
            anyhow::bail!("tags must start with + to add or - to remove, got '{}'", change);
        }
    }
    Ok(TagChanges { add, remove })
}

fn main() -> Result<()> {

    let conn = open_db()
        .context("Failed to open database")?;
    db::init_db(&conn)
        .context("Failed to initialize database")?;

    let task_list = TaskRepo::new(conn);

    let cli = Cli::parse();

    let command = cli.command.unwrap_or(Command::List);

    match command {
        Command::Add { name } => {
            let mut task = Task::new(name);
            let id = task_list.add(&task)
                .context("Failed to add task")?;
            task.id = Some(id);
            print_task_list(&task_list)?;
            Ok(())
        }
        Command::List => {
            print_task_list(&task_list)?;
            Ok(())
        },
        Command::Doing {id} => {
            task_list.set_status(id, TaskStatus::Doing)?;

            print_task_list(&task_list)?;
            Ok(())
        },

        Command::Done {id} => {
            task_list.set_status(id, TaskStatus::Done)?;

            print_task_list(&task_list)?;
            Ok(())
        },

        Command::Todo { id } => {
            task_list.set_status(id, TaskStatus::Todo)?;

            print_task_list(&task_list)?;
            Ok(())
        }
        Command::Delete { id } => {
            task_list.delete(id)
                .context("Failed to delete task")?;

            print_task_list(&task_list)?;
            Ok(())

        }
        Command::Tag { task_id, changes } => {
            let tag_changes = parse_tag_changes(&changes)?;

            for tag in &tag_changes.add {
                task_list.add_tag(&task_id, tag)?;
            }
            for tag in &tag_changes.remove {
                task_list.remove_tag(&task_id, tag)?;
            }

            Ok(())

        }
    }
}

