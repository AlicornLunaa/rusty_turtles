use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use futures_util::{SinkExt, StreamExt};
use tokio::{sync::{Mutex, mpsc, oneshot}, task::JoinHandle};
use tokio_tungstenite::tungstenite::Message;

use crate::{gateway::{ServerAction, ServerMessage}, turtle::{TurtleAction, queries::{TurtleInit, TurtleQuery}, types::*}, util::{name_generator::generate_random_name, script}};

/// A struct representing a turtle
pub struct Turtle {
    id: u64,
    turtle_write_stream: mpsc::Sender<TurtleMessage>,
    gateway_write_stream: mpsc::Sender<ServerMessage>,
    join_handle: JoinHandle<()>,
    x: i64,
    y: i64,
    z: i64,
    direction: Direction,
    valid: Arc<Mutex<bool>>
}

impl Turtle {
    // Constructors
    fn spawn_worker_thread(client_id: u64, mut turtle_rx: mpsc::Receiver<TurtleMessage>, server_tx: mpsc::Sender<ServerMessage>, mut ws_sender: TurtleSink, mut ws_receiver: TurtleSource, background_valid_flag: Arc<Mutex<bool>>) -> JoinHandle<()> {
        // This function spawns a background thread and takes full ownership of the websocket stream
        tokio::spawn(async move {
            // This background thread manages messages from tx by reading rx and sending it to the socket
            println!("Started consumer thread");

            let mut pending_responses = HashMap::new();
            let mut request_id: u64 = 0;

            loop {
                tokio::select! {
                    Some(message) = turtle_rx.recv() => {
                        // This means the server has sent a message to this turtle, which should be relayed to the websocket
                        match message {
                            TurtleMessage::Action{ actions, return_tx } => {
                                // This is an action request, meaning it will tell the turtle a bunch of stuff to do, then get back a success or fail
                                let payload = TurtlePayload::Procedure { id: request_id, data: actions };
                                let payload = serde_json::to_string(&payload).unwrap();

                                if let Err(e) = ws_sender.send(Message::Text(payload.into())).await {
                                    eprintln!("{e}");
                                    break;
                                }

                                if let Some(return_tx) = return_tx {
                                    // Then wait for the response
                                    pending_responses.insert(request_id, return_tx);
                                }

                                request_id += 1;
                            },
                            TurtleMessage::Query{ query, response } => {
                                // This is an query request, meaning it will ask the turtle something complex and return the complex data
                                let payload = TurtlePayload::Query { id: request_id, data: query };
                                let payload = serde_json::to_string(&payload).unwrap();

                                if ws_sender.send(Message::Text(payload.into())).await.is_err() {
                                    break;
                                }

                                // Then wait for the response
                                pending_responses.insert(request_id, response);
                                request_id += 1;
                            },
                        }
                    },
                    Some(message) = ws_receiver.next() => {
                        // This means the turtle has sent a message to the server/websocket, which should be relayed to the gateway or used as a response
                        match message {
                            Ok(Message::Text(message)) => {
                                // First decapsulate the data
                                let payload: TurtlePayload = serde_json::from_str(&message).unwrap();

                                match payload {
                                    TurtlePayload::Query { id, data } => {
                                        // This is a new request for the server from the turtle. This one is blocking for the turtle worker thread
                                        // but only because the turtle doesn't parallelize the runtime
                                        let server_action: ServerAction = serde_json::from_value(data).unwrap();
                                    
                                        // Create a channel to get the result
                                        let (tx, rx) = oneshot::channel::<Result<Value, String>>();
                                        let server_message = ServerMessage::Procedure { client_id: client_id, action: server_action, tx };
                                        
                                        if let Err(e) = server_tx.send(server_message).await {
                                            // Something went wrong with the write stream
                                            let response = TurtleResponse { success: false, reason: Some(e.to_string()), last_action: 0, data: None };
                                            let payload = TurtlePayload::Response { id, data: response };
                                            let payload = serde_json::to_string(&payload).unwrap();

                                            if ws_sender.send(Message::Text(payload.into())).await.is_err() {
                                                break;
                                            }
                                        }

                                        // Wait for immediate response
                                        let response = match rx.await {
                                            Ok(Ok(data)) => TurtlePayload::Response { id, data: TurtleResponse { success: true, reason: None, last_action: 0, data: Some(data) } },
                                            Ok(Err(e)) => TurtlePayload::Response { id, data: TurtleResponse { success: false, reason: Some(e), last_action: 0, data: None } },
                                            Err(e) => TurtlePayload::Response { id, data: TurtleResponse { success: false, reason: Some(e.to_string()), last_action: 0, data: None } },
                                        };
                                        let response = serde_json::to_string(&response).unwrap();

                                        if ws_sender.send(Message::Text(response.into())).await.is_err() {
                                            break;
                                        }
                                    },
                                    TurtlePayload::Response { id, data } => {
                                        // This is a response from the turtle to some sort of pending request
                                        let sender = pending_responses.remove(&id);

                                        if let Some(sender) = sender {
                                            // Send the response to the sender if the channel still exists
                                            if sender.send(data).is_err() {
                                                // If the response channel doesn't work, we've probably dropped the turtle.
                                                break;
                                            }
                                        }
                                    },
                                    _ => {
                                        panic!("Turtles are not allowed to execute this payload.");
                                    }
                                }
                            },
                            _ => {
                                break;
                            }
                        }
                    }
                }
            }

            *background_valid_flag.lock().await = false;
            println!("Consumer lost, closing thread");
        })
    }

    async fn initial_handshake(turtle_tx: mpsc::Sender<TurtleMessage>) -> Result<(i64, i64, i64, Direction), String> {
        // Get a fresh turtle script in case there was an update
        let (version, new_script) = script::read_turtle_script();

        // Send a message to the turtle's websocket
        let (tx, rx) = oneshot::channel::<TurtleResponse>();
        let message = TurtleMessage::Query { query: TurtleInit { version, script: new_script }.to_payload(), response: tx };

        if let Err(e) = turtle_tx.send(message).await {
            // Something went wrong with the write stream
            return Err(e.to_string());
        }

        // Wait for the response
        match rx.await {
            Ok(response) => {
                let data = response.data.unwrap();

                match serde_json::from_value(data) {
                    Ok(data) => Ok(data),
                    Err(e) => Err("Error with turtle setup. ".to_string() + &e.to_string()),
                }
            },
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                Err(e.to_string())
            },
        }
    }

    pub async fn new(id: u64, ws_stream: TurtleSocket, server_tx: mpsc::Sender<ServerMessage>) -> Result<Self, String> {
        // Obtain turtle information via questioning
        let (ws_sender, ws_receiver) = ws_stream.split();
        let (turtle_tx, turtle_rx) = mpsc::channel::<TurtleMessage>(32);
        let valid = Arc::new(Mutex::new(true));

        let handle = Self::spawn_worker_thread(id, turtle_rx, server_tx.clone(), ws_sender, ws_receiver, Arc::clone(&valid));
        let (x, y, z, direction) = Self::initial_handshake(turtle_tx.clone()).await?;
        
        // Return the turtle object which has the sender object too
        let mut turtle = Turtle {
            id,
            turtle_write_stream: turtle_tx,
            gateway_write_stream: server_tx,
            join_handle: handle,
            x, y, z, direction,
            valid
        };
        let _ = turtle.execute(TurtleAction::ChangeName { name: generate_random_name() }).await;

        Ok(turtle)
    }

    // Turtle getters
    pub async fn is_valid(&self) -> bool {
        *self.valid.lock().await
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_position(&self) -> (i64, i64, i64) {
        (self.x, self.y, self.z)
    }

    pub fn get_direction(&self) -> Direction {
        self.direction.clone()
    }

    pub fn get_block_ahead(&self) -> (i64, i64, i64) {
        // Returns the location of the block ahead
        let mut x = self.x;
        let mut z = self.z;

        match self.get_direction() {
            Direction::NORTH => z -= 1,
            Direction::EAST => x += 1,
            Direction::SOUTH => z += 1,
            Direction::WEST => x -= 1,
        }

        (x, self.y, z)
    }

    pub(super) fn get_server_tx(&self) -> &mpsc::Sender<ServerMessage> {
        &self.gateway_write_stream
    }

    // Private helpers for communication with the actual turtle
    async fn get_state_from_actions(&mut self, actions: &Vec<TurtleAction>, last_action: usize) -> Result<(), TurtleError> {
        // Builds the positions iteratively to the last action
        let (mut x, mut y, mut z) = self.get_position();
        let mut direction = self.get_direction();

        for i in 0..last_action {
            let current_action = actions.get(i).unwrap();

            match current_action {
                TurtleAction::Forward => {
                    match direction {
                        Direction::NORTH => z -= 1,
                        Direction::SOUTH => z += 1,
                        Direction::EAST => x += 1,
                        Direction::WEST => x -= 1,
                    }
                },
                TurtleAction::Back => {
                    match direction {
                        Direction::NORTH => z += 1,
                        Direction::SOUTH => z -= 1,
                        Direction::EAST => x -= 1,
                        Direction::WEST => x += 1,
                    }
                },
                TurtleAction::Up => {
                    y += 1;
                },
                TurtleAction::Down => {
                    y -= 1;
                },
                TurtleAction::TurnLeft => {
                    direction = match direction {
                        Direction::NORTH => Direction::WEST,
                        Direction::WEST => Direction::SOUTH,
                        Direction::SOUTH => Direction::EAST,
                        Direction::EAST => Direction::NORTH,
                    };
                },
                TurtleAction::TurnRight => {
                    direction = match direction {
                        Direction::NORTH => Direction::EAST,
                        Direction::EAST => Direction::SOUTH,
                        Direction::SOUTH => Direction::WEST,
                        Direction::WEST => Direction::NORTH,
                    };
                },
                _ => {} // Ignore all actions which are not movement based
            }
        }

        // Save the data
        self.x = x;
        self.y = y;
        self.z = z;
        self.direction = direction.clone();

        // Send data to the turlte as an update
        let (tx, rx) = oneshot::channel::<TurtleResponse>();
        let message = TurtleMessage::Action { actions: vec![TurtleAction::UpdateLocation { x, y, z, direction }], return_tx: Some(tx) };

        if let Err(e) = self.turtle_write_stream.send(message).await {
            // Something went wrong with the write stream
            *self.valid.lock().await = false;
            return Err(TurtleError::SocketError(e.to_string()));
        }

        // Wait for the response
        match rx.await {
            Ok(_) => Ok(()),
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                *self.valid.lock().await = false;
                Err(TurtleError::SocketError(e.to_string()))
            },
        }
    }

    pub async fn oneshot(&self, action: TurtleAction) -> Result<(), TurtleError> {
        let message = TurtleMessage::Action { actions: vec![action], return_tx: None };

        if let Err(e) = self.turtle_write_stream.send(message).await {
            // Something went wrong with the write stream
            return Err(TurtleError::SocketError(e.to_string()));
        }

        Ok(())
    }

    pub async fn execute(&mut self, action: TurtleAction) -> Result<TurtleResponse, TurtleError> {
        self.execute_batch(vec![action]).await
    }

    pub async fn execute_batch(&mut self, actions: Vec<TurtleAction>) -> Result<TurtleResponse, TurtleError> {
        // Send a message to the turtle's websocket
        let (tx, rx) = oneshot::channel::<TurtleResponse>();
        let message = TurtleMessage::Action { actions: Vec::clone(&actions), return_tx: Some(tx) };

        if let Err(e) = self.turtle_write_stream.send(message).await {
            // Something went wrong with the write stream
            *self.valid.lock().await = false;
            return Err(TurtleError::SocketError(e.to_string()));
        }

        // Wait for the response
        match rx.await {
            Ok(response) => {
                // Update turtle's saved position
                self.get_state_from_actions(&actions, response.last_action as usize).await?;
                Ok(response)
            },
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                *self.valid.lock().await = false;
                Err(TurtleError::SocketError(e.to_string()))
            },
        }
    }

    pub async fn query<Q: TurtleQuery>(&self, query: Q) -> Result<Q::Response, TurtleError> {
        // Send a message to the turtle's websocket
        let (tx, rx) = oneshot::channel::<TurtleResponse>();
        let message = TurtleMessage::Query { query: query.to_payload(), response: tx };

        if let Err(e) = self.turtle_write_stream.send(message).await {
            // Something went wrong with the write stream
            *self.valid.lock().await = false;
            return Err(TurtleError::SocketError(e.to_string()));
        }

        // Wait for the response
        match rx.await {
            Ok(response) => Ok(serde_json::from_value(response.data.unwrap()).unwrap()),
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                *self.valid.lock().await = false;
                Err(TurtleError::SocketError(e.to_string()))
            },
        }
    }
}

impl Drop for Turtle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

/// Simple helper to build action lists for a turtle
pub struct TurtleSequence<'a> {
    turtle: &'a mut Turtle,
    queue: Vec<TurtleAction>,
}

impl<'a> TurtleSequence<'a> {
    pub fn new(turtle: &'a mut Turtle) -> Self {
        Self { turtle, queue: Vec::new() }
    }

    pub fn forward(mut self) -> Self {
        self.queue.push(TurtleAction::Forward);
        self
    }

    pub fn back(mut self) -> Self {
        self.queue.push(TurtleAction::Back);
        self
    }

    pub fn turn_right(mut self) -> Self {
        self.queue.push(TurtleAction::TurnRight);
        self
    }
    
    pub fn turn_left(mut self) -> Self {
        self.queue.push(TurtleAction::TurnLeft);
        self
    }

    pub fn up(mut self) -> Self {
        self.queue.push(TurtleAction::Up);
        self
    }

    pub fn down(mut self) -> Self {
        self.queue.push(TurtleAction::Down);
        self
    }

    // The only async function is the one that fires the payload
    pub async fn execute(self) -> Result<u64, TurtleError> {
        match self.turtle.execute_batch(self.queue).await {
            Ok(response) => Ok(response.last_action),
            Err(err) => Err(err),
        }
    }
}