# todo-app

Simple todo app made to test the already acquired knowledge and learn sth new along the way

Client-side:
It's responsible for interaction with the user, collection of data such as user input, making requests to the handler and presenting handler responses to the user

Server-side (Salvo.rs):
Start-up responsibilities:
- creation of database connection pool
- creating a router and starting a server

Request handling:
- listening on the port for requests
- extraction of data sent with requests
- using it to conduct operations on the database
- forming and sending responses to the client

All the handler functions operate on the same principle which could be defined as a list of tasks:
1. Get the data from the request body and parse it into a HashMap
2. Assign this data to variables for code simplicity
3. Execute database operations
4. Check the results of the operations
5. Respond to Ok's and Err's accordingly
