use core::fmt;

use rusqlite::Result;
use rusqlite::{params, Connection};

/// Attempt to get a connection to the SQLite database
///
/// `path` specifies which file to open for DB read/write.
/// If `path` is ":memory:", then the DB is created in-memory.
pub fn get_connection(path: &str) -> Result<Connection> {
    if path == ":memory:" {
        Connection::open_in_memory()
    } else {
        Connection::open(path)
    }
}

/// A Todo object from the database
#[derive(Debug)]
pub struct Todo {
    /// Database primary key of the todo object
    pub id: i64,
    /// Title of the todo
    pub title: String,
    /// Description of the todo's objective
    pub description: String,
    /// True if complete, false otherwise
    pub complete: bool,
}

impl Todo {
    /// Create a new todo item and insert it in the database
    ///
    /// ID field will remain 0 until `self.save()` is called. Afterwards,
    /// it is updated with the ID from the database.
    pub fn new(title: &str, description: &str, complete: bool) -> Self {
        Todo {
            id: 0,
            title: title.to_string(),
            description: description.to_string(),
            complete,
        }
    }

    /// Save a Todo to the database and return the updated-and-inserted version
    pub fn save(self, conn: &Connection) -> Result<Self> {
        let mut stmt =
            conn.prepare("INSERT INTO todos(title, description, complete) VALUES(?1, ?2, ?3);")?;
        let todo_id = stmt.insert(params![self.title, self.description, self.complete])?;
        Ok(Todo {
            id: todo_id,
            title: self.title,
            description: self.description,
            complete: self.complete,
        })
    }

    /// Get a todo from an ID in the database
    pub fn from_id(id: i64, conn: &Connection) -> Result<Todo> {
        let todo = conn.query_row(
            "SELECT id, title, description, complete FROM todos WHERE id = ?",
            [id],
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
}

impl PartialEq for Todo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.title == other.title && self.description == other.description
    }
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
{} (#{} - {}completed):
    Description: {:?}",
            self.title.as_str(),
            self.id,
            if self.complete { "" } else { "not " },
            self.description,
        )
    }
}

/// Initialize the database tables if they do not exist
pub fn initialize_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL UNIQUE,
                    description TEXT,
                    complete BOOLEAN DEFAULT false
                );",
        (),
    )?;

    Ok(())
}

/// Get all of the todos in the database
pub fn get_all_todos(conn: &Connection) -> Result<Vec<Todo>> {
    let mut stmt = conn.prepare("SELECT id, title, description, complete FROM todos")?;
    let rows = stmt.query_map([], |row| {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            complete: row.get(3)?,
        })
    })?;

    let mut todos = Vec::new();
    for todo in rows {
        todos.push(todo?);
    }

    Ok(todos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Result<Connection> {
        let conn = get_connection(":memory:")?;
        initialize_tables(&conn)?;
        Ok(conn)
    }

    #[test]
    fn db_initializes() {
        assert_eq!(setup().is_ok(), true);
    }

    #[test]
    fn make_and_get_a_todo() -> Result<()> {
        let conn = setup()?;

        let todo = Todo::new("Test todo", "This is a test description", false).save(&conn)?;

        // assert that the todo returned and the one saved are equal
        assert_eq!(todo, Todo::from_id(todo.id, &conn)?);

        Ok(())
    }

    #[test]
    fn get_all_todos_at_once() -> Result<()> {
        let conn = setup()?;

        let my_todos = vec![
            Todo::new("Test 1", "First test", false).save(&conn)?,
            Todo::new("Test 2", "Seconds test", false).save(&conn)?,
            Todo::new("Test 3", "Another test", true).save(&conn)?,
            Todo::new("Test 4", "Fourth test", false).save(&conn)?,
        ];

        let todos = get_all_todos(&conn)?;

        assert_eq!(todos.len(), my_todos.len());

        Ok(())
    }
}
