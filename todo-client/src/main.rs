use std::io::{self};
use reqwest::Error;
use serde::{Deserialize, Serialize};



#[derive(Deserialize, Serialize, Debug)]
struct Todo {
    id: i16,
    name: String,
    description: String,
    done: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = reqwest::Client::new();

    loop {
        println!("Enter command (GET, GET NAME, POST, UPDATE, MD, DELETE) or EXIT to quit:");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read line");

        match command.trim().to_uppercase().as_str() {
            "GET" => {
                let res = client.get("http://127.0.0.1:7878/display")
                    .send()
                    .await?
                    .json::<Vec<Todo>>()
                    .await?;
                
                for todo in res {
                    println!("ID: {}\nName: {}\nContent:\n{}\n\nDone: {}", todo.id, todo.name, todo.description, todo.done);
                }
                
                
            },
            "GET NAME" => {
                let mut todo_name = String::new();
                loop {
                    println!("Your todo name: ");
                    io::stdin().read_line(&mut todo_name).expect("Something isn't right about the name of your todo");
                    todo_name = todo_name.trim().to_string();

                    println!("Your todo name: {}\n\nIs that correct? (Y/n)", todo_name);
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer).expect("Wrong answer");
                    
                    if answer.trim().to_ascii_uppercase() == "Y" && !todo_name.is_empty() {
                        break;
                    }
                    else if todo_name.is_empty() {
                        println!("todo name can not be empty");
                    }

                    todo_name.clear();
                }

                let res = client.get("http://127.0.0.1:7878/display/display_one")
                    .json(&serde_json::json!({"name": todo_name}))
                    .send()
                    .await?
                    .text()
                    .await?;
                
                println!("Response: {}", res);
                
            },
            "POST" => {
                let mut todo_name = String::new();
                let mut todo_desc = String::new();
                loop {
                    println!("Your new todo name: ");
                    io::stdin().read_line(&mut todo_name).expect("Something isn't right about the name of your future todo");
                    todo_name = todo_name.trim().to_string();

                    println!("{}'s content/description: ", todo_name);
                    io::stdin().read_line(&mut todo_desc).expect("Something isn't right about the content of your future todo");
                    todo_desc = todo_desc.trim().to_string();

                    println!("New Todo contents:\nName: {}\nDescription: {}\n\nIs that correct? (Y/n)", todo_name, todo_desc);
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer).expect("Wrong answer");
                    
                    if answer.trim().to_ascii_uppercase() == "Y" && !todo_name.is_empty() {
                        break;
                    }
                    else if todo_name.is_empty() {
                        println!("todo name can not be empty");
                    }

                    todo_name.clear();
                    todo_desc.clear();
                }



                let res = client.post("http://127.0.0.1:7878/create")
                    .json(&serde_json::json!({"name": todo_name,"description": todo_desc}))
                    .send()
                    .await?
                    .text()
                    .await?;
                println!("Response: {}", res);
            },
            "MD" => {
                let mut todo_id = String::new();

                loop {
                    println!("Todo id: ");
                    io::stdin().read_line(&mut todo_id).expect("Something isn't right about the id given");
                    todo_id = todo_id.trim().to_string();

                    println!("Todo id: {}\n\nIs that correct? (Y/n)", todo_id);
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer).expect("Wrong answer");
                    
                    if answer.trim().to_ascii_uppercase() == "Y" && !todo_id.is_empty() {
                        break;
                    }
                    else if todo_id.is_empty() {
                        println!("todo id can not be empty");
                    }

                    todo_id.clear();
                }

                let res = client.patch("http://127.0.0.1:7878/md")
                    .json(&serde_json::json!({"id": todo_id}))
                    .send()
                    .await?
                    .text()
                    .await?;

                println!("{}", res);
            },
            "DELETE" => {
                let mut todo_id = String::new();

                loop {
                    println!("Todo id: ");
                    io::stdin().read_line(&mut todo_id).expect("Something isn't right about the id given");
                    todo_id = todo_id.trim().to_string();

                    println!("Todo id: {}\n\nIs that correct? (Y/n)", todo_id);
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer).expect("Wrong answer");
                    
                    if answer.trim().to_ascii_uppercase() == "Y" && !todo_id.is_empty() {
                        break;
                    }
                    else if todo_id.is_empty() {
                        println!("todo id can not be empty");
                    }

                    todo_id.clear();
                }


                let res = client.delete("http://127.0.0.1:7878/delete")
                    .json(&serde_json::json!({"id": todo_id}))
                    .send()
                    .await?
                    .text()
                    .await?;
                
                println!("Response: {}", res);
            },
            "UPDATE" => {
                let mut todo_id = String::new();
                let mut todo_name = String::new();
                let mut todo_desc = String::new();
                let mut todo_done = "false";

                loop {
                    println!("Enter the id of the todo you want to update: ");
                    io::stdin().read_line(&mut todo_id).expect("Something isn't right about the id given");
                    todo_id = todo_id.trim().to_string();

                    println!("Todo id: {}\n\nIs that correct? (Y/n)", todo_id);
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer).expect("Wrong answer");
                    
                    if answer.trim().to_ascii_uppercase() == "Y" {
                        let check_result = client.get("http://127.0.0.1:7878/update")
                            .json(&serde_json::json!({"id": todo_id}))
                            .send()
                            .await?
                            .text()
                            .await?;

                        println!("{}", check_result);
                        break;
                    }

                    todo_id.clear();
                }

                loop {
                    println!("Enter the new name of the todo: ");
                    io::stdin().read_line(&mut todo_name).expect("Something isn't right about the name given");
                    todo_name = todo_name.trim().to_string();
            
                    println!("{}'s description: ", todo_desc);
                    io::stdin().read_line(&mut todo_desc).expect("Something isn't right about the description given");
                    todo_desc = todo_desc.trim().to_string();
            
                    println!("Todo id: {}\nTodo name: {}\nTodo description: {}\nInput 'YMD' to mark as done\n\nIs that correct? (Y/YMD/n)", todo_id, todo_name, todo_desc);
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer).expect("Wrong answer");
                    
                    if answer.trim().to_ascii_uppercase() == "Y" {
                        todo_name = todo_name.trim().to_string();
                        todo_desc = todo_desc.trim().to_string();
                        break;
                    }
                    else if answer.trim().to_ascii_uppercase() == "YMD" {
                        todo_name = todo_name.trim().to_string();
                        todo_desc = todo_desc.trim().to_string();
                        todo_done = "true";
                        break;
                    }
            
                    todo_name.clear();
                    todo_desc.clear();
                }



                let res = client.put("http://127.0.0.1:7878/update")
                    .json(&serde_json::json!({"id": todo_id,"name": todo_name, "desc": todo_desc, "done": todo_done}))
                    .send()
                    .await?
                    .text()
                    .await?;
                
                println!("Response: {}", res);
            },
            "EXIT" => break,
            _ => println!("Unknown command"),
        }

        println!();
    }

    Ok(())
}
