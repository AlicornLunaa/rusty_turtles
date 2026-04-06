use serde_json::Value;
use simple_websockets::{Event, Message, Responder};
use std::{collections::HashMap, env};

use agent::AgentType;

use crate::agent::{TurtleMessage, TurtleOpCode};

mod blocks;
mod object_relations;
mod agent;

const DEFAULT_PORT: u16 = 8080;

fn broadcast_message(sender: u64, message: &Message, clients: &HashMap<u64, Responder>) {
    // Iterate through all connected clients and send the message to each one except the sender
    for (client_id, responder) in clients.iter() {
        if *client_id != sender {
            responder.send(message.clone());
        }
    }
}

fn handle_payload(text: &str) -> TurtleMessage {
    let msg: Vec<Value> = serde_json::from_str(text).unwrap();

    let opcode = msg.get(0)
        .and_then(Value::as_u64)
        .expect("opcode must be a number");

    let opcode = TurtleOpCode::try_from(opcode)
        .expect("unknown opcode");

    match opcode {
        TurtleOpCode::UpdatePosition => {
            let x = msg.get(1).and_then(Value::as_i64).unwrap();
            let y = msg.get(2).and_then(Value::as_i64).unwrap();
            let z = msg.get(3).and_then(Value::as_i64).unwrap();
            println!("update position: {}, {}, {}", x, y, z);
            TurtleMessage::PositionUpdate { x, y, z }
        },
        TurtleOpCode::UpdateRotation => {
            let rotation = msg.get(1).and_then(Value::as_i64).unwrap() as i8;
            println!("rotate: {}", rotation);
            TurtleMessage::RotationUpdate { rotation }
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

fn main() {
    // Initialize the database connection and create the necessary tables
    let database = create_database();

    // Read the default port from the environment variable, or use 8080 if not set
    let port = match env::var("SERVER_PORT") {
        Ok(val) => val.parse::<u16>().unwrap_or(DEFAULT_PORT),
        Err(_) => DEFAULT_PORT,
    };
    println!("Starting server on port {}", port);

    // Start the WebSocket server on the specified port and obtain a map of connected clients
    let event_hub = simple_websockets::launch(port).expect(&format!("Failed to launch on port {}", port));
    let mut waiting_room: HashMap<u64, Responder> = HashMap::new(); // A temporary map to hold clients until their agent type is determined
    let mut clients: HashMap<u64, Responder> = HashMap::new();
    let mut temp_agent_type: HashMap<u64, AgentType> = HashMap::new();

    // Enter the main event loop to handle incoming WebSocket events
    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}, determining agent type...", client_id);
                waiting_room.insert(client_id, responder);
            },
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                waiting_room.remove(&client_id);
                clients.remove(&client_id);
                temp_agent_type.remove(&client_id);
            },
            Event::Message(client_id, message) => {
                // Check if the client is still in the waiting room (i.e., we haven't determined their agent type yet)
                if waiting_room.contains_key(&client_id) {
                    println!("Received a message from client #{} in waiting room: {:?}", client_id, message);

                    let responder = waiting_room.remove(&client_id).unwrap();
                    let agent_type_string = match message {
                        Message::Text(text) => text,
                        _ => {
                            println!("Invalid message type from client #{}: expected text, got {:?}", client_id, message);
                            continue;
                        }
                    }.trim().to_lowercase();

                    match agent_type_string.as_str() {
                        "turtle" => {
                            println!("Client #{} identified as a turtle agent.", client_id);
                            clients.insert(client_id, responder);
                            temp_agent_type.insert(client_id, AgentType::Turtle);
                        },
                        "client" => {
                            println!("Client #{} identified as a client agent.", client_id);
                            clients.insert(client_id, responder);
                            temp_agent_type.insert(client_id, AgentType::Client);
                        },
                        _ => {
                            println!("Unknown agent type '{}' from client #{}. Disconnecting.", agent_type_string, client_id);
                            responder.send(Message::Text("unknown_agent_type".to_string()));
                            responder.close();
                        }
                    }
                } else if clients.contains_key(&client_id) {
                    println!("Received a message from client #{}: {:?}", client_id, message);
                    let responder = clients.get(&client_id).unwrap();
                    let agent_type = temp_agent_type.get(&client_id).unwrap();

                    // Handle the message based on the agent type and broadcast it to other clients
                    match agent_type {
                        AgentType::Turtle => {
                            let raw_data = match message {
                                Message::Text(text) => text,
                                _ => {
                                    println!("Invalid message type from client #{}: expected text, got {:?}", client_id, message);
                                    continue;
                                }
                            };
                            let turtle_message = handle_payload(raw_data.trim());
                            println!("Processed turtle message from client #{}: {:?}", client_id, turtle_message);
                        },
                        AgentType::Client => {
                            // For client agents, we can implement different message handling logic if needed
                            println!("Received a message from a client agent #{}: {:?}", client_id, message);
                            responder.send(message);
                        }
                    }
                }
            },
        }
    }
}