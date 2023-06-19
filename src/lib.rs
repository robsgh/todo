use rusqlite::Result;
use rusqlite::{params, Connection};

/// A Todo object from the database
#[derive(Debug)]
pub struct Todo {
    /// Database primary key of the todo object
    pub id: i32,
    /// Title of the todo
    pub title: String,
    /// Description of the todo's objective
    pub description: String,
    /// True if complete, false otherwise
    pub complete: bool,
}

/// A client for the Todo database
#[derive(Debug)]
pub struct TodoClient {
    /// DB Connection object which will be opened after bulding
    conn: Connection,
}

/// Attempt to get a connection to the SQLite database
///
/// `path` specifies which file to open for DB read/write.
/// If `path` is ":memory:", then the DB is created in-memory.
fn get_connection(path: &str) -> Result<Connection> {
    if path == ":memory:" {
        Connection::open_in_memory()
    } else {
        Connection::open(path)
    }
}

/// Create table(s) in the database
fn create_tables(conn: &Connection) -> Result<()> {
    let query = "CREATE TABLE IF NOT EXISTS todos(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL UNIQUE,
                    description TEXT,
                    complete BOOLEAN DEFAULT false
                );";
    conn.execute(query, ())?;

    Ok(())
}

impl TodoClient {
    /// Build a todos client and initialize all tables
    pub fn build(path: &str) -> Result<Self> {
        let conn = get_connection(path)?;
        create_tables(&conn)?;

        Ok(Self { conn })
    }

    /// Add a todo to the todos DB
    pub fn add(&self, todo: &Todo) -> Result<()> {
        let mut statement = self
            .conn
            .prepare("INSERT INTO todos(title, description, complete) VALUES (?1, ?2, ?3);")?;

        statement.execute(params![todo.title, todo.description, todo.complete])?;

        Ok(())
    }

    /// Get a todo by its DB ID
    pub fn get_todo_by_id(&self, todo_id: i32) -> Result<Todo> {
        let todo = self.conn.query_row(
            "SELECT id, title, description, complete FROM todos WHERE id = ?",
            [todo_id],
            |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    complete: row.get(3)?,
                })
            },
        )?;

        Ok(todo)
    }

    /// Get all todos in the DB
    pub fn get_all_todos(&self) -> Result<Vec<Todo>> {
        let mut statement = self
            .conn
            .prepare("SELECT id, title, description, complete FROM todos")?;
        let todos_iter = statement.query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                complete: row.get(3)?,
            })
        })?;

        let mut todos: Vec<Todo> = Vec::new();
        for todo in todos_iter {
            todos.push(todo.unwrap());
        }

        Ok(todos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_the_todos() {
        let client = TodoClient::build(":memory:").unwrap();
        client
            .add(&Todo {
                id: 0,
                title: "Test todo numero uno".to_string(),
                description: "This is the description of the test todo".to_string(),
                complete: false,
            })
            .unwrap();
        assert_eq!(1, client.get_all_todos().unwrap().len());
    }

    #[test]
    fn get_a_specific_todo() {
        let client = TodoClient::build(":memory:").unwrap();
        client.add(&Todo {
                id: 0,
                title: "Test todo numero uno".to_string(),
                description: "This is the description of the test todo".to_string(),
                complete: false,
            })
            .unwrap();
        assert_eq!(1, client.get_todo_by_id(1).unwrap().id);
    }
}
