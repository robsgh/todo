use tasker::{Todo, TodoClient};

fn main() {
    let client = TodoClient::build(":memory:").unwrap();

    for i in 0..100 {
        let todo = Todo {
            id: i,
            title: format!("Todo number {}", i + 1),
            description: "Placeholder desc".to_string(),
            complete: false,
        };

        client.add(&todo).unwrap();
    }

    let todos = client.get_all_todos().unwrap();

    for todo in todos {
        println!(
            "===============================
            Task (id: {:?}, complete: {:?}): {:?}
            Description: {:?}
            \n",
            todo.id, todo.complete, todo.title, todo.description,
        );
    }
}
