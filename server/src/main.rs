use simple_websockets::{Event, Message, Responder};
use std::{collections::HashMap, env};
mod agent_type;

const DEFAULT_PORT: u16 = 8080;

struct Block {
    x: i32,
    y: i32,
    z: i32,
    block_type: String,
}

fn broadcast_message(sender: u64, message: &Message, clients: &HashMap<u64, Responder>) {
    // Iterate through all connected clients and send the message to each one except the sender
    for (client_id, responder) in clients.iter() {
        if *client_id != sender {
            responder.send(message.clone());
        }
    }
}

fn main() {
    // Read the database path from the environment variable, or use in memory if not set
    let db_conn = match env::var("DATABASE_URL") {
        Ok(val) => {
            println!("Using database at path: {}", val);
            rusqlite::Connection::open(val).expect("Failed to open database")
        },
        Err(_) => {
            println!("No database path provided, using in-memory database.");
            rusqlite::Connection::open_in_memory().expect("Failed to open in-memory database")
        },
    };

    // Read the default port from the environment variable, or use 8080 if not set
    let port = match env::var("SERVER_PORT") {
        Ok(val) => val.parse::<u16>().unwrap_or(DEFAULT_PORT),
        Err(_) => DEFAULT_PORT,
    };
    println!("Starting server on port {}", port);

    // Start the WebSocket server on the specified port and obtain a map of connected clients
    let event_hub = simple_websockets::launch(port).expect(&format!("Failed to launch on port {}", port));
    let mut clients: HashMap<u64, Responder> = HashMap::new();

    // Enter the main event loop to handle incoming WebSocket events
    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}, determining agent type...", client_id);
                clients.insert(client_id, responder);
            },
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                clients.remove(&client_id);
            },
            Event::Message(client_id, message) => {
                println!("Received a message from client #{}: {:?}", client_id, message);

                let responder = clients.get(&client_id).unwrap();
                responder.send(message);
            },
        }
    }
}