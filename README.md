# Simple Rust Todo API

Project made to test the already acquired knowledge and learn sth new along the way.

A backend application for managing Todo items, built with Rust, Salvo web framework, and SQLx for PostgreSQL database interaction. 

## Server-Side (Salvo.rs Backend)

This is the core of the application, handling API requests for Todo management.

### Responsibilities:

*   **Initialization:**
    *   Reads the `DATABASE_URL` environment variable.
    *   Establishes an asynchronous connection pool to the PostgreSQL database using SQLx.
    *   Initializes and manages the database pool as a shared static resource (`once_cell`).
    *   Defines API routes using the Salvo router.
    *   Starts the web server and listens for incoming connections on `127.0.0.1:7878`.
*   **Request Handling:**
    *   Listens for HTTP requests on the configured port.
    *   Routes incoming requests to the appropriate handler function based on the path and HTTP method.
    *   Parses request data (query parameters, JSON bodies).
    *   Interacts with the PostgreSQL database via SQLx to perform CRUD operations on `todos` table.
    *   Handles potential database errors and request parsing errors gracefully.
    *   Formats responses as JSON, indicating success or failure and including relevant data or error messages.

### Technologies Used:

*   **Rust:** The core programming language.
*   **Salvo:** A high-performance async web framework for Rust.
*   **SQLx:** An async, compile-time checked SQL toolkit for Rust.
*   **Tokio:** An asynchronous runtime for Rust.
*   **Serde:** A framework for serializing and deserializing Rust data structures efficiently (used for JSON).
*   **PostgreSQL:** The relational database used for storing todo items.

### API Endpoints:

*   `GET /todos`: Retrieves all todo items.
*   `POST /todos`: Creates a new todo item.
    *   *Body:* `{ "name": "string", "description": "string" }`
*   `GET /todos/todo?id=<id>`: Retrieves a single todo item by its ID.
*   `PUT /todos/todo?id=<id>`: Updates an existing todo item (replaces all fields).
    *   *Body:* `{ "name": "string", "description": "string", "done": boolean }`
*   `PATCH /todos/todo?id=<id>`: Marks a specific todo item as done.
*   `DELETE /todos/todo?id=<id>`: Deletes a todo item by its ID.


## Client-Side (Outdated)
Latest commit does not include any updates for the client, mostly because I have plans to create a new frontend client in JS

It's responsible for interaction with the user, collection of data such as user input, making requests to the handler and presenting handler responses to the user
