use std::{collections::HashMap, sync::{Arc, Mutex}};

use salvo::prelude::*;
use serde::{Serialize, Deserialize};
use sqlx::{prelude::FromRow, PgPool};
use once_cell::sync::OnceCell;

mod pool_sqlx;

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
struct Todo {
    id: i16,
    name: String,
    description: String,
    done: bool,
}

struct OperationLock { // TODO not implemented yet
    mutex_lock: Mutex<()>
}

static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::new();

#[tokio::main]
async fn main() {
    // Get the database URL from the environment variables
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(k) => k,
        Err(e) => match e {
            std::env::VarError::NotPresent | std::env::VarError::NotUnicode(_) => {
                println!("Environment variable (with database url): {:?}", e);
                return;
            },
        }
    };
    
    // Establish a connection to the database
    let pool = match pool_sqlx::establish_connection(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            println!("Error connecting to database: {:?}", e);
            return;
        }
    };

    // Create an Arc-wrapped pool and set it in the DB_POOL static variable
    let pool = Arc::new(pool);
    let _ = DB_POOL.set(pool);

    // Create the routes for different endpoints
    let create = Router::with_path("create").hoop(create_check).post(create_todo);
    let display = Router::with_path("display").get(display_todo).push(Router::with_path("display_one").get(display_one));
    let update = Router::with_path("update").get(update_check).put(update_todo);
    let md = Router::with_path("md").hoop(md_check).patch(md_todo);
    let delete = Router::with_path("delete").hoop(delete_check).delete(delete_todo);
    
    // Create a router and add the routes to it
    let router = Router::new()
        .push(create)
        .push(display)
        .push(md)
        .push(delete)
        .push(update);

    // Start the server and bind it to the specified address
    Server::new(TcpListener::new("127.0.0.1:7878").bind().await).serve(router).await;
}

pub fn get_postgres() -> &'static PgPool {
    DB_POOL.get().unwrap()
}

#[handler]
async fn display_todo(res: &mut Response) {
    let sql_select_all = "SELECT id, name, description, done FROM todos";

    let data = sqlx::query_as::<_, Todo>(&sql_select_all).fetch_all(get_postgres()).await.unwrap();

    res.render(Json(data));
}

#[handler]
async fn display_one(req: &mut Request, res: &mut Response) {
    // Parse the JSON data from the request into a `HashMap<String, String>`.
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();

    // Retrieve the value of the "name" key from the parsed data, which represents the name of the todo item.
    let todo_name = requested_data.get("name").unwrap();

    // Define the SQL query string to select the todo item with the given name from the database.
    let sql_select_one = "SELECT id, name, description, done FROM todos WHERE name = $1";

    // Execute the SQL query using `sqlx::query_as` and bind the todo name as a parameter.
    match sqlx::query_as::<_, Todo>(&sql_select_one)
        .bind(todo_name)
        .fetch_one(get_postgres())
        .await
    {
        Ok(data) => {
            // If the query execution is successful (`Ok(data)`), format the retrieved todo item data into a string.
            let temp = format!(
                "\n\nID: {}\nName: {}\nContent:\n{}\n\nDone: {}",
                data.id, data.name, data.description, data.done
            );
            res.render(temp);
        }
        Err(e) => {
            // If an error occurs during the query execution (`Err(e)`), print the error message and set the response status code to 500 (Internal Server Error).
            eprintln!("An error occurred: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[handler]
async fn create_todo(req: &mut Request, res: &mut Response) {
    // SQL query string to insert a new todo item into the database
    let sql_insert = "INSERT INTO todos (name, description) VALUES ($1,$2)";

    // Parse the JSON payload from the request into a `HashMap<String, String>`
    let request_data = req.parse_json::<HashMap<String, String>>().await.unwrap();

    // Extract the `name` and `description` values from the `HashMap`
    let todo_name = request_data.get("name").unwrap();
    let todo_desc = request_data.get("description").unwrap();

    // Execute the SQL query with the bound parameters using the `get_postgres` function to get the database connection 
    let data = sqlx::query(sql_insert)
        .bind(&todo_name)
        .bind(&todo_desc)
        .execute(get_postgres())
        .await;

    // Check if the query execution is successful
    match data {
        // If successful, render a "Created todo..." message as the response
        Ok(_) => res.render("Created todo..."),

        // If there is an error during query execution, format an error message with the error details and render it as the response
        Err(e) => {
            let response = format!("An error occurred: {}", e);
            res.render(response);
        }
    }
}

// an async function called `create_check` that handles a request to check if a todo item with a specific name already exists in the database.

#[handler]
async fn create_check(req: &mut Request, res: &mut Response) {
    // SQL query to select the "id" column from the "todos" table where the "name" column matches the provided name
    let sql_select = "SELECT id FROM todos WHERE name = $1";
    
    // Parsing the JSON body of the request into a HashMap
    let request_data = req.parse_json::<HashMap<String, String>>().await.unwrap();
    
    // Retrieving the value of the "name" field from the request data
    let todo_name = request_data.get("name").unwrap();

    // Executing the SQL query using the `sqlx` library, binding the name value as a parameter
    let data = sqlx::query(&sql_select)
        .bind(&todo_name)
        .fetch_optional(get_postgres())
        .await;

    // Handling the result of the query execution
    match data {
        // If the query returns no rows, a todo item with the given name does not exist
        Ok(None) => {},
        
        // If the query returns a row, a todo item with the given name already exists
        Ok(Some(_)) => {
            // Rendering the response with the message "Todo of that name already exists"
            res.render("Todo of that name already exists");
            
            // Setting the status code to `StatusCode::BAD_REQUEST`
            res.status_code(StatusCode::BAD_REQUEST);
        },
        
        // If there is an error during the query execution, printing an error message
        Err(e) => eprintln!("An error occurred: {}", e),
    }
}

// The rest of the handlers work on the same principle as those above

#[handler]
async fn md_todo(req: &mut Request, res: &mut Response) {
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();
    let todo_id: i16 = requested_data.get("id").unwrap().parse().unwrap();

    let sql_mark_done = "UPDATE todos SET done = true WHERE id = $1";

    let _ = sqlx::query(&sql_mark_done).bind(todo_id).execute(get_postgres()).await;

    let sql_md_select = "SELECT * FROM todos WHERE id = $1 AND done = true";

    match sqlx::query_as::<_, Todo>(&sql_md_select).bind(todo_id).fetch_optional(get_postgres()).await {
        Ok(Some(data)) => {
            res.render(format!("ID: {}\nName: {}\nContent:\n{}\n\nDone: {}\n\nTodo successfully modified", data.id, data.name, data.description, data.done));
        },
        Ok(None) => {
            res.render("Couldn't update todo");
            res.status_code(StatusCode::BAD_REQUEST);
        },
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }}

#[handler]
async fn md_check(req: &mut Request, res: &mut Response) {
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();
    let todo_id: i16 = requested_data.get("id").unwrap().parse().unwrap();

    let sql_check_exists = "SELECT id FROM todos WHERE (id = $1)";
    let data = sqlx::query(&sql_check_exists).bind(todo_id).fetch_optional(get_postgres()).await;

    if let Ok(None) = data {
        res.render("Todo does not exist");
        res.status_code(StatusCode::BAD_REQUEST);
    }
    else if let Err(e) = data {
        eprintln!("Error occurred: {}", e);
    }
}

#[handler]
async fn delete_todo(req: &mut Request, res: &mut Response) {
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();
    let todo_id: i16 = requested_data.get("id").unwrap().parse().unwrap();

    
    let sql_drop_todo = "DELETE FROM todos WHERE id = $1".to_string();

    let _ = sqlx::query(&sql_drop_todo).bind(&todo_id).execute(get_postgres()).await;

    let sql_check_success = "SELECT * FROM todos WHERE id = $1";

    let data = sqlx::query(&sql_check_success).bind(&todo_id).fetch_optional(get_postgres()).await;
    

    if let Ok(None) = data {
        let deleted = format!("Todo with id: {} succesfully deleted", todo_id);
        res.render(deleted);
    } else if let Ok(Some(row)) = data {
        let deleted_fail = format!("Todo with id: {} still exists", todo_id);
        res.render(deleted_fail);
    } else if let Err(e) = data {
        eprintln!("An error occurred: {}", e);
    }
}

#[handler]
async fn delete_check(req: &mut Request, res: &mut Response) {
    let sql_delete_check = "SELECT id FROM todos WHERE id = $1";
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();
    let todo_id: i16 = requested_data.get("id").unwrap().parse().unwrap();

    let data = sqlx::query(&sql_delete_check).bind(todo_id).fetch_optional(get_postgres()).await;

    if let Ok(None) = data {
        res.render("Todo does not exist");
        res.status_code(StatusCode::BAD_REQUEST);
    }
    else if let Err(e) = data {
        eprintln!("Error occurred: {}", e);
    }
}


#[handler]
async fn update_todo(req: &mut Request, res: &mut Response) {
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();

    let todo = Todo {
        id: requested_data.get("id").unwrap().parse().unwrap(),
        name: requested_data.get("name").unwrap().to_string(),
        description: requested_data.get("desc").unwrap().to_string(),
        done: requested_data.get("done").unwrap().clone().parse().unwrap(),
    };
    

    let sql_command_update = "UPDATE todos SET name = $1, description = $2, done = $3 WHERE id = $4";

    let data = sqlx::query(&sql_command_update).bind(&todo.name).bind(&todo.description).bind(&todo.done).bind(&todo.id).execute(get_postgres()).await;

    if let Err(e) = data {
        println!("{}", e);
    }
    else {
        let sql_confirm = "SELECT id, name, description, done FROM todos WHERE id = $1";
        let updated_todo = sqlx::query_as::<_, Todo>(&sql_confirm).bind(&todo.id).fetch_one(get_postgres()).await.unwrap();
        let updated = format!("Updated Todo\n\nId: {}\nName: {}\nDescription: {}\nDone: {}", updated_todo.id, updated_todo.name, updated_todo.description, updated_todo.done);
        res.render(updated);
    }
        
}

#[handler]
async fn update_check(req: &mut Request, res: &mut Response) {
    let requested_data = req.parse_json::<HashMap<String, String>>().await.unwrap();
    let todo_id: i16 = requested_data.get("id").unwrap().parse().unwrap();
    
    let sql_update_check = "SELECT id, name, description, done FROM todos WHERE id = $1";

    let data = sqlx::query_as::<_, Todo>(&sql_update_check).bind(todo_id).fetch_optional(get_postgres()).await;

    if let Ok(None) = data {
        let not_existent = "Such todo does not exist";
        res.render(not_existent);
        res.status_code(StatusCode::BAD_REQUEST);
    } else if let Ok(Some(row)) = &data {
        let data = data.unwrap().unwrap();
        let todo_found = format!("Todo found!\n\nID: {}\nName: {}\nContent:\n{}\n\nDone: {}", data.id, data.name, data.description, data.done);
        res.render(todo_found);
        res.status_code(StatusCode::OK);
    } else if let Err(e) = data {
        eprintln!("An error occurred: {}", e);
    }
}
