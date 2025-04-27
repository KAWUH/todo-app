use std::{collections::HashMap, sync::{Arc, Mutex}};

use backend_error::BackendError;
use salvo::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::json;
use sqlx::{prelude::FromRow, PgPool};
use once_cell::sync::OnceCell;

mod pool_sqlx;
mod backend_error;

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
struct Todo {
    id: i32,
    name: String,
    description: String,
    done: bool,
}

struct OperationLock { // TODO not implemented yet
    mutex_lock: Mutex<()>
}

static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), BackendError> {
    // Get the database URL from the environment variables
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(k) => k,
        Err(e) => return Err(BackendError::EnvError(format!("Environment variable DATABASE_URL fetching returned: {:?}", e)))
    };
    
    // Establish a connection to the database
    let pool = pool_sqlx::establish_connection(&database_url).await?;

    // Create an Arc-wrapped pool and set it in the DB_POOL static variable
    DB_POOL.set(
        Arc::new(pool)
    ).unwrap();

    // Create a router and add the routes to it
    let router = Router::with_path("todos")
        .get(display_todos)
        .post(create_todo)
        .push(
            Router::with_path("todo")
                .get(display_one)
                .delete(delete_todo)    
                .put(update_todo)
                .patch(md_todo)
        );

    // Start the server and bind it to the specified address
    Server::new(TcpListener::new("127.0.0.1:7878").bind().await).serve(router).await;

    Ok(())
}

pub fn get_postgres() -> &'static PgPool {
    DB_POOL.get().unwrap()
}

#[handler]
async fn display_todos(res: &mut Response) {   

    match sqlx::query_as::<_, Todo>("SELECT id, name, description, done FROM todos")
        .fetch_all(get_postgres())
        .await
    {
        Ok(data) => {
            res.render(Json(json!({
                "success": true,
                "todos": data
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
        }
    }
}

#[handler]
async fn display_one(req: &mut Request, res: &mut Response) {
    
    // Extract the "id" parameter from the request URL
    let todo_id = match req.query::<i32>("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing 'id' query parameter"
            })));
            return;
        }
    };

    // Execute the SQL query to fetch the todo item with the given id from the database
    match sqlx::query_as::<_, Todo>("SELECT id, name, description, done FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_optional(get_postgres())
        .await
    {
        Ok(Some(todo)) => {
            res.render(Json(json!({
                "success": true,
                "todo": todo
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(json!({
                "success": false,
                "error": format!("Todo with id '{}' not found", todo_id)
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
        }
    }
}

#[handler]
async fn create_todo(req: &mut Request, res: &mut Response) {

    // Parse the JSON payload from the request into a `HashMap<String, String>`
    let request_data = match req.parse_json::<HashMap<String, String>>().await {
        Ok(data) => data,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Invalid JSON payload"
            })));
            return;
        }
    };
    
    // Extract the `name` value from the request_data
    let todo_name = match request_data.get("name") {
        Some(name) if !name.trim().is_empty() => name,
        _ => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing or empty 'name' field"
            })));
            return;
        }
    };

    // Check if a todo with the same name already exists
    match sqlx::query("SELECT 1 FROM todos WHERE name = $1")
        .bind(todo_name)
        .fetch_optional(get_postgres())
        .await
    {
        Ok(Some(_)) => {
            res.status_code(StatusCode::CONFLICT);
            res.render(Json(json!({
                "success": false,
                "error": "Todo with that name already exists"
            })));
            return;
        },
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
            return;
        },
        Ok(None) => {}
    }

    // Extract the `description` value from the request_data
    let todo_desc = match request_data.get("description") {
        Some(desc) => desc,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing 'description' field"
            })));
            return;
        }
    };

    // Insert the new todo and return it
    match sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (name, description) VALUES ($1, $2) RETURNING id, name, description, done"
    )
    .bind(todo_name)
    .bind(todo_desc)
    .fetch_one(get_postgres())
    .await
    {
        Ok(todo) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(json!({
                "success": true,
                "todo": todo
            })));
        },
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
        }
    }
}

#[handler]
async fn md_todo(req: &mut Request, res: &mut Response) {
    
    // Extract the "id" parameter from the request URL
    let todo_id = match req.query::<i32>("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing 'id' query parameter"
            })));
            return;
        }
    };

    // Check if the todo exists
    match sqlx::query("SELECT 1 FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_optional(get_postgres())
        .await
    {
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(json!({
                "success": false,
                "error": format!("Todo with id {} does not exist", todo_id)
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
            return;
        }
        Ok(Some(_)) => {}
    }

    // Mark the todo as done and fetch it
    match sqlx::query_as::<_, Todo>(
        "UPDATE todos SET done = true WHERE id = $1 RETURNING id, name, description, done"
    )
    .bind(todo_id)
    .fetch_optional(get_postgres())
    .await
    {
        Ok(Some(todo)) => {
            res.render(Json(json!({
                "success": true,
                "todo": todo
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": "Todo was marked as done but could not be retrieved"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
        }
    }
}

#[handler]
async fn delete_todo(req: &mut Request, res: &mut Response) {
    
    // Extract the "id" parameter from the request URL
    let todo_id = match req.query::<i32>("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing 'id' query parameter"
            })));
            return;
        }
    };

    // Check if the todo exists
    match sqlx::query("SELECT 1 FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_optional(get_postgres())
        .await
    {
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(json!({
                "success": false,
                "error": format!("Todo with id {} does not exist", todo_id)
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
            return;
        }
        Ok(Some(_)) => {}
    }

    // Delete the todo
    match sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(todo_id)
        .execute(get_postgres())
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 1 {
                res.render(Json(json!({
                    "success": true,
                    "message": format!("Todo with id {} successfully deleted", todo_id)
                })));
            } else {
                // Rare but possible case
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(json!({
                    "success": false,
                    "error": "Todo was not deleted due to an unknown error"
                })));
            }
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
        }
    }
}

#[handler]
async fn update_todo(req: &mut Request, res: &mut Response) {

    // Extract the "id" parameter from the request URL
    let todo_id = match req.query::<i32>("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing 'id' query parameter"
            })));
            return;
        }
    };

    // Check if the todo exists
    match sqlx::query("SELECT 1 FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_optional(get_postgres())
        .await
    {
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(json!({
                "success": false,
                "error": format!("Todo with id {} does not exist", todo_id)
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
            return;
        }
        Ok(Some(_)) => {}
    }

    // Parse the JSON payload from the request into a HashMap
    let request_data = match req.parse_json::<HashMap<String, String>>().await {
        Ok(data) => data,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Invalid JSON payload"
            })));
            return;
        }
    };

    // Extract and validate the "name" field
    let name = match request_data.get("name") {
        Some(name) if !name.trim().is_empty() => name,
        _ => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing or empty 'name' field"
            })));
            return;
        }
    };

    // Extract the "description" field
    let description = match request_data.get("description") {
        Some(desc) => desc,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing 'description' field"
            })));
            return;
        }
    };

    // Extract and parse the "done" field
    let done: bool = match request_data.get("done").and_then(|d| d.parse().ok()) {
        Some(done) => done,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({
                "success": false,
                "error": "Missing or invalid 'done' field"
            })));
            return;
        }
    };

    // Update the todo in the database and fetch it
    match sqlx::query_as::<_, Todo>(
        "UPDATE todos SET name = $1, description = $2, done = $3 WHERE id = $4 RETURNING id, name, description, done"
    )
    .bind(name)
    .bind(description)
    .bind(done)
    .bind(todo_id)
    .fetch_optional(get_postgres())
    .await
    {
        Ok(Some(updated_todo)) => {
            res.render(Json(json!({
                "success": true,
                "todo": updated_todo
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": "Todo was not updated due to an unknown error"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })));
        }
    }
}
