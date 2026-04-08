use std::sync::Arc;
use std::{env, rc::Rc};
use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::managers::block_manager::BlockManager;
use crate::managers::turtle_manager::TurtleManager;
use crate::object_relations::ORM;
use crate::turtle::Turtle;

mod object_relations;
mod managers;
mod turtle;

const DEFAULT_PORT: u16 = 8080;

pub enum AgentType {
    Turtle,
    Client,
}

pub struct AppState {
    pub block_manager: BlockManager,
    pub database: Rc<ORM>,
}
impl AppState {
    pub fn new(database: ORM) -> Self {
        let database = Rc::new(database);

        AppState {
            block_manager: BlockManager::new(database.clone()),
            database: database,
        }
    }
}

fn create_database() -> object_relations::ORM {
    // Read the database path from the environment variable, or use in memory if not set
    let database_url = match env::var("DATABASE_URL") {
        Ok(val) => {
            println!("Using database at path: {}", val);
            Some(val)
        },
        Err(_) => {
            println!("No database path provided, using in-memory database.");
            None
        },
    };

    let database = object_relations::ORM::new(database_url);
    database.create_tables().expect("Failed to create database tables");
    database
}

async fn create_socket_server() -> TcpListener {
    // Read the default port from the environment variable, or use 8080 if not set
    let port = match env::var("SERVER_PORT") {
        Ok(val) => val.parse::<u16>().unwrap_or(DEFAULT_PORT),
        Err(_) => DEFAULT_PORT,
    };
    println!("Starting server on port {}", port);

    // Start the WebSocket server on the specified port and obtain a map of connected clients
    // simple_websockets::launch(port).expect(&format!("Failed to launch on port {}", port))
    let listener: TcpListener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind to port 8080");
    listener
}

#[tokio::main]
async fn main() {
    // Initialize the database and the WebSocket server
    let mut app_state = AppState::new(create_database());
    let turtle_manager = Arc::new(Mutex::new(TurtleManager::new()));
    let listener = create_socket_server().await;

    // Main loop to accept incoming connections and spawn a new task for each one
    loop {
        let (stream, addr) = listener.accept().await.expect("Failed to accept connection");
        let turtle_manager = Arc::clone(&turtle_manager);

        println!("New client connected from {}, determining type", addr);

        tokio::spawn(async move {
            // Accept the WebSocket connection and split it into a sender and receiver
            let mut ws_stream = tokio_tungstenite::accept_async(stream).await.expect("Failed to accept WebSocket connection");
            println!("WebSocket connection established with {}", addr);

            if let Some(response) = ws_stream.next().await {
                match response {
                    Ok(message) => {
                        // Simple text answer, either "turtle" or "client" for now.
                        match message.to_text().unwrap() {
                            "turtle" => {
                                let turtle = Turtle::new(ws_stream).await.unwrap();
                                turtle_manager.lock().await.add_turtle(turtle);
                            },
                            "client" => todo!(),
                            _ => {
                                eprintln!("Failed to select correct agent");
                                return;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to initialize client. {e}");
                    },
                }
            }
        });
    }
}