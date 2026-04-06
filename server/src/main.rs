use std::{collections::HashMap, env};
use simple_websockets::{Event, EventHub, Message, Responder};
use serde_json::Value;

use crate::agent::{AgentType, PositionUpdate, RotationUpdate, TurtleMessage, TurtleOpCode};
use crate::client_manager::ClientManager;

mod blocks;
mod object_relations;
mod client_manager;
mod turtle;
mod agent;

const DEFAULT_PORT: u16 = 8080;

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

fn create_socket_server() -> EventHub {
    // Read the default port from the environment variable, or use 8080 if not set
    let port = match env::var("SERVER_PORT") {
        Ok(val) => val.parse::<u16>().unwrap_or(DEFAULT_PORT),
        Err(_) => DEFAULT_PORT,
    };
    println!("Starting server on port {}", port);

    // Start the WebSocket server on the specified port and obtain a map of connected clients
    simple_websockets::launch(port).expect(&format!("Failed to launch on port {}", port))
}

fn broadcast_message(sender: u64, message: &Message, clients: &HashMap<u64, Responder>) {
    // Iterate through all connected clients and send the message to each one except the sender
    for (client_id, responder) in clients.iter() {
        if *client_id != sender {
            responder.send(message.clone());
        }
    }
}

fn handle_payload(text: &str) -> Box<dyn TurtleMessage> {
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
            Box::new(PositionUpdate { x, y, z })
        },
        TurtleOpCode::UpdateRotation => {
            let rotation = msg.get(1).and_then(Value::as_i64).unwrap() as i8;
            println!("rotate: {}", rotation);
            Box::new(RotationUpdate { rotation })
        }
    }
}

fn main() {
    // Initialize the database and the WebSocket server
    let database = create_database();
    let event_hub = create_socket_server();

    // Create a client manager to keep track of connected clients and their agent types
    let mut client_manager = ClientManager::new();

    // Enter the main event loop to handle incoming WebSocket events
    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}, determining agent type...", client_id);
                client_manager.add_to_waiting_room(client_id, responder);
            },
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                client_manager.remove_client(client_id);
            },
            Event::Message(client_id, message) => {
                // Check if the client is still in the waiting room (i.e., we haven't determined their agent type yet)
                if client_manager.is_waiting(client_id) {
                    println!("Received a message from client #{} in waiting room: {:?}", client_id, message);

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
                            client_manager.qualify_agent(client_id, AgentType::Turtle).unwrap();
                        },
                        "client" => {
                            println!("Client #{} identified as a client agent.", client_id);
                            client_manager.qualify_agent(client_id, AgentType::Client).unwrap();
                        },
                        _ => {
                            println!("Unknown agent type '{}' from client #{}. Disconnecting.", agent_type_string, client_id);
                            client_manager.send(client_id, Message::Text("unknown_agent_type".into())).unwrap();
                            client_manager.remove_client(client_id);
                        }
                    }
                } else {
                    println!("Received a message from client #{}: {:?}", client_id, message);
                    let agent_type = client_manager.get_agent_type(client_id);

                    // Handle the message based on the agent type and broadcast it to other clients
                    match agent_type {
                        Some(AgentType::Turtle) => {
                            let raw_data = match message {
                                Message::Text(text) => text,
                                _ => {
                                    println!("Invalid message type from client #{}: expected text, got {:?}", client_id, message);
                                    continue;
                                }
                            };
                            let turtle_message = handle_payload(raw_data.trim());
                            let res = turtle_message.handle_message(client_id, &mut client_manager, &database);

                            if let Err(err) = res {
                                println!("Error handling message from client #{}: {}", client_id, err);
                                continue;
                            }
                        },
                        Some(AgentType::Client) => {
                            // For client agents, we can implement different message handling logic if needed
                            println!("Received a message from a client agent #{}: {:?}", client_id, message);
                            client_manager.send(client_id, message).unwrap();
                        }
                        None => {
                            println!("Received a message from an unidentified client #{}. Ignoring.", client_id);
                        },
                    }
                }
            },
        }
    }
}