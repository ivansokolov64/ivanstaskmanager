use anyhow::{Context, Result};
use crate::db::{open_db, TaskRepo};
use clap::{Parser, Subcommand};
use crate::task::{Task, TaskStatus};

pub mod task;
pub mod db;

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
    Doing {
        id: String
    },
    Done {
        id: String
    }
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
            println!("{task}");
            Ok(())
        }
        Command::List => {
            let tasks = task_list.list()
                .context("Failed to get task list")?;

            println!("ALL TASKS ({})", tasks.len());
            for task in tasks {
                println!("{task}");
            }
            Ok(())
        },
        Command::Doing {id} => {
            task_list.set_status(id, TaskStatus::Doing)?;

            let tasks = task_list.list()
                .context("Failed to get task list")?;

            println!("ALL TASKS ({})", tasks.len());
            for task in tasks {
                println!("{task}");
            }
            Ok(())
        },

        Command::Done {id} => {
            task_list.set_status(id, TaskStatus::Done)?;

            let tasks = task_list.list()
                .context("Failed to get task list")?;

            println!("ALL TASKS ({})", tasks.len());
            for task in tasks {
                println!("{task}");
            }
            Ok(())
        },


    }

}

