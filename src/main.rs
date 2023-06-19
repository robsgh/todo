use std::io;
use std::{fs, io::Write};

use clap::{Args, Parser, Subcommand};
use rtd::{Todo, TodoClient};

use directories::ProjectDirs;

#[derive(Debug, Parser)]
#[command(name = "rtd")]
#[command(about = "Rob's todo CLI")]
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

    /// Get a todo from the database
    #[command(arg_required_else_help = true)]
    Get(GetArgs),
}

#[derive(Debug, Args)]
struct GetArgs {
    #[command(subcommand)]
    command: Option<GetCommands>,
}

#[derive(Debug, Subcommand)]
enum GetCommands {
    /// ID of the todo to retreive
    Id { id: i32 },
    /// Get all of the todos
    All,
}

fn get_client() -> rusqlite::Result<TodoClient> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "arknet", "rtd") {
        fs::create_dir_all(proj_dirs.data_dir())
            .expect("homedir should exist and be owned by the current user");

        let db_path = proj_dirs.data_dir().join("db.sqlite3");
        let client = TodoClient::build(
            db_path
                .to_str()
                .expect("path to DB file should be valid unicode"),
        )?;

        Ok(client)
    } else {
        println!("WARNING: could not load the todo DB, loading in-memory DB");

        Ok(TodoClient::build(":memory")?)
    }
}

fn print_todo(todo: &Todo) {
    println!(
        "{} (id:{:?}, complete:{:?}):
    Description: {:?}",
        todo.title.as_str(), todo.id, todo.complete,todo.description, 
    );
}

fn print_all_todos() {
    let client = get_client().expect("client should be able to be created");
    let todos = client
        .get_all_todos()
        .expect("rtd should be able to list all todos");

    for todo in todos {
        print_todo(&todo);
    }
}

fn print_todo_by_id(id: i32) {
    let client = get_client().expect("client should be able to be created");
    let maybe_todo = match client.get_todo_by_id(id) {
        Ok(todo) => Some(todo),
        _ => {
            println!("Could not find any todo with id: {id}");
            None
        }
    };

    if let Some(todo) = maybe_todo {
        print_todo(&todo);
    }
}

fn create_new_todo(title: Option<String>) {
    let client = get_client().expect("client should be able to be created");

    let mut title_str = String::new();
    let mut description_str = String::new();

    if let Some(t) = title {
        title_str = t;
    } else {
        print!("Todo title: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut title_str)
            .expect("should be able to read title from stdin");
    }

    print!("Todo Description: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut description_str)
        .expect("should be able to read description from stdin");

    let todo = Todo {
        id: -1,
        title: title_str.trim().to_string(),
        description: description_str.trim().to_string(),
        complete: false,
    };

    client
        .add(&todo)
        .expect("TodoClient shoudl be able to add a todo");

    println!("Created new todo - {:?}", todo.title);
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::New { title } => {
            create_new_todo(title);
        }
        Commands::Get(get) => {
            let get_cmd = get.command.unwrap_or(GetCommands::All);
            match get_cmd {
                GetCommands::All => {
                    print_all_todos();
                }
                GetCommands::Id { id } => {
                    print_todo_by_id(id);
                }
            }
        }
    }
}
