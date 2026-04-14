use futures_util::{SinkExt, StreamExt};
use shared::blocks::BlockNotification;
use tokio::{net::TcpStream, sync::{broadcast, mpsc}, task::JoinHandle};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{client::ClientMessage, gateway::{ServerAction, ServerMessage}};

pub struct Client {
    id: u64,
    client_tx: mpsc::Sender<ClientMessage>,
    gateway_write_stream: mpsc::Sender<ServerMessage>,
    join_handle: JoinHandle<()>,
}

impl Client {
    fn spawn_worker_thread(client_id: u64, mut client_rx: mpsc::Receiver<ClientMessage>, server_tx: mpsc::Sender<ServerMessage>, ws: WebSocketStream<TcpStream>, mut notifs: broadcast::Receiver<BlockNotification>) -> JoinHandle<()> {
        // Thisction spawns a background thread and takes full ownership of the websocket stream
        tokio::spawn(async move {
            // This background thread manages messages from tx by reading rx and sending it to the socket
            println!("Started consumer thread");

            let (mut ws_sender, mut ws_receiver) = ws.split();

            loop {
                tokio::select! {
                    Some(message) = client_rx.recv() => {
                        // This means the server has sent a message to this client, which should be relayed to the websocket
                        let payload = serde_json::to_string(&message).unwrap();

                        if ws_sender.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    },
                    Some(message) = ws_receiver.next() => {
                        // This means the client has sent a message to the server/websocket, which should be relayed to the gateway or used as a response
                        match message {
                            Ok(Message::Text(message)) => {
                                // First decapsulate the data
                                let server_action: ServerAction = serde_json::from_str(&message).unwrap();
                                
                                if let Err(_) = server_tx.send(ServerMessage::Oneshot { client_id, action: server_action }).await {
                                    // Something went wrong with the write stream
                                    break;
                                }
                            },
                            _ => {
                                break;
                            }
                        }
                    },
                    Ok(message) = notifs.recv() => {
                        println!("{:?}", message);
                    }
                }
            }

            println!("Consumer lost, closing thread");
        })
    }

    pub fn new(id: u64, ws_stream: WebSocketStream<TcpStream>, server_tx: mpsc::Sender<ServerMessage>, notifs: broadcast::Receiver<BlockNotification>) -> Self {
        let (client_tx, client_rx) = mpsc::channel::<ClientMessage>(32);
        let handle = Self::spawn_worker_thread(id, client_rx, server_tx.clone(), ws_stream, notifs);
        
        Self {
            id,
            client_tx,
            gateway_write_stream: server_tx,
            join_handle: handle,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub(super) fn get_server_tx(&self) -> &mpsc::Sender<ServerMessage> {
        &self.gateway_write_stream
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}