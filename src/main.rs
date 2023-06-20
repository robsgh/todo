use std::{io, io::Write, path::PathBuf};

use clap::{Parser, Subcommand};
use rtd::{get_all_todos, get_connection, initialize_tables, Todo};

use directories::ProjectDirs;

#[derive(Debug, Parser)]
#[command(name = "rtd", about = "Rob's todo CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new todo item
    New {
        /// Title of the todo to create
        title: Option<String>,
    },

    /// Get one or more todos from the database
    #[command(arg_required_else_help = true)]
    Get {
        /// ID of the todo
        id: Option<i64>,

        /// Get all of the todos in the database
        #[arg(short, long)]
        all: bool,
    },
}

/// Get the path to the DB file on disk
fn get_db_file_path() -> PathBuf {
    let proj_dir = ProjectDirs::from("com", "arknet", "rtd")
        .expect("to be able to set a project directory in a homedir");
    proj_dir.data_dir().join("db.sqlite3")
}

/// Setup the database
fn setup() -> rusqlite::Result<()> {
    let conn = get_connection(
        get_db_file_path()
            .to_str()
            .expect("db path should be valid"),
    )?;
    initialize_tables(&conn)?;

    Ok(())
}

fn print_all_todos() {
    let conn = get_connection(
        get_db_file_path()
            .to_str()
            .expect("db path should be valid"),
    )
    .expect("connection with DB should be established");

    match get_all_todos(&conn) {
        Ok(todos) => {
            for todo in todos {
                println!("{todo}")
            }
        }
        Err(e) => panic!("An error ocurred while querying the todos DB: {:?}", e),
    }
}

fn print_todo_from_id(id: i64) {
    let conn = get_connection(
        get_db_file_path()
            .to_str()
            .expect("db path should be valid"),
    )
    .expect("connection with DB should be established");

    match Todo::from_id(id, &conn) {
        Ok(todo) => println!("{todo}"),
        Err(_) => {
            println!("Could not find any todo with id: {id}");
        }
    };
}

fn create_new_todo(todo_title: Option<String>) {
    let conn = get_connection(
        get_db_file_path()
            .to_str()
            .expect("db path should be valid"),
    )
    .expect("connection with DB should be established");

    let mut title = String::new();
    let mut description = String::new();

    // get the title if we don't have one already
    if let None = todo_title {
        print!("Todo title: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut title)
            .expect("should be able to read title from stdin");
    } else {
        title = todo_title.unwrap();
    }

    print!("Todo Description: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut description)
        .expect("should be able to read description from stdin");

    let todo = Todo::new(title.trim(), description.trim(), false)
        .save(&conn)
        .expect("todo should be saved in DB");

    println!("Created new todo:");
    println!("===========================================");
    println!("{todo}");
    println!("===========================================");
}

fn main() {
    let args = Cli::parse();

    setup().expect("database should setup properly");

    match args.command {
        Commands::New { title } => {
            create_new_todo(title);
        }
        Commands::Get { id, all } => {
            if id.is_some() {
                print_todo_from_id(id.unwrap());
            } else if all {
                print_all_todos();
            }
        }
    }
}
