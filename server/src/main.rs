use std::collections::HashSet;
use std::sync::Arc;
use std::env;
use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::client::Client;
use crate::gateway::Gateway;
use crate::managers::block_manager::BlockManager;
use crate::managers::client_manager::ClientManager;
use crate::managers::path_manager::PathManager;
use crate::managers::turtle_manager::TurtleManager;
use crate::turtle::Turtle;
use crate::util::vector::Vector3;

mod managers;
mod gateway;
mod turtle;
mod client;
mod util;

const DEFAULT_PORT: u16 = 8080;

async fn create_socket_server() -> TcpListener {
    // Read the default port from the environment variable, or use 8080 if not set
    let port = match env::var("SERVER_PORT") {
        Ok(val) => val.parse::<u16>().unwrap_or(DEFAULT_PORT),
        Err(_) => DEFAULT_PORT,
    };
    println!("Starting server on port {}", port);

    // Start the WebSocket server on the specified port and obtain a map of connected clients
    let listener: TcpListener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind to port 8080");
    listener
}

#[tokio::main]
async fn main() {
    // Initialize the database and the WebSocket server
    let client_manager = ClientManager::new();
    let turtle_manager = TurtleManager::new();
    let block_manager = BlockManager::new().await;
    let gateway = Gateway::new(turtle_manager.clone(), block_manager.clone());
    let mut planner = PathManager::new(block_manager.clone(), turtle_manager.clone());

    let listener = create_socket_server().await;

    // Testing loop
    tokio::spawn({
        let turtle_manager = turtle_manager.clone();
        
        async move {
            // Spawn a thread which every 10 seconds spawns a thread to communicate with turtles
            loop {
                let turtles_to_remove = Arc::new(Mutex::new(HashSet::new()));

                // Start a future action
                planner.set_goal(0, Vector3::new(-12, 56, -4));
                planner.set_goal(1, Vector3::new(-8, 56, -1));
                planner.set_goal(2, Vector3::new(-6, 56, -7));
                planner.execute().await;
                planner.set_goal(0, Vector3::new(-12, 56, 3));
                planner.set_goal(1, Vector3::new(-19, 56, -1));
                planner.set_goal(2, Vector3::new(-18, 56, 4));
                let results = planner.execute().await;

                for (i, result) in results.iter().enumerate() {
                    println!("Turtle {i} path: {result:?}");
                }
                println!();

                for turtle in turtle_manager.iter_turtles().await {
                    // Make sure the turtle is valid
                    {
                        let turtle_lock = turtle.lock().await;

                        if !turtle_lock.is_valid().await {
                            turtles_to_remove.lock().await.insert(turtle_lock.get_id());
                            continue;
                        }
                    }
                }

                {
                    // Remove turtles marked as invalid
                    let mut turtles_to_remove = turtles_to_remove.lock().await;
        
                    for i in turtles_to_remove.iter() {
                        println!("Removing turtle {i}");
                        turtle_manager.remove_turtle(*i).await;
                    }
        
                    turtles_to_remove.clear();
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    // Main loop to accept incoming connections and spawn a new task for each one
    loop {
        let (stream, addr) = listener.accept().await.expect("Failed to accept connection");
        let block_manager = block_manager.clone();
        let turtle_manager = turtle_manager.clone();
        let client_manager = client_manager.clone();
        let server_write_stream = gateway.get_sender();

        println!("New client connected from {}, determining type", addr);

        tokio::spawn(async move {
            // Accept the WebSocket connection and split it into a sender and receiver
            let mut ws_stream = tokio_tungstenite::accept_async(stream).await.expect("Failed to accept WebSocket connection");
            print!("WebSocket connection established with {} ", addr);

            if let Some(response) = ws_stream.next().await {
                match response {
                    Ok(message) => {
                        // Simple text answer, either "turtle" or "client" for now.
                        match message.to_text().unwrap().trim().to_lowercase().as_str() {
                            "turtle" => {
                                println!("it's a turtle");
                                let new_turtle_id = turtle_manager.get_next_id().await;
                                let turtle = Turtle::new(new_turtle_id, ws_stream, server_write_stream).await.unwrap();
                                let turtle = Arc::new(Mutex::new(turtle));
                                turtle_manager.add_turtle(turtle).await;
                            },
                            "client" => {
                                println!("it's a client");
                                let new_client_id = client_manager.get_next_id().await;
                                let client = Client::new(new_client_id, ws_stream, server_write_stream, block_manager.subscribe());
                                let client = Arc::new(Mutex::new(client));
                                client_manager.add_client(client).await;
                            },
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
