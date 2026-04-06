use macroquad::prelude::*;
use tungstenite::{connect, Message};
use url::Url;

#[macroquad::main("vTurtle Client")]
async fn main() {
    // Connect to a WebSocket server
    let (mut socket, response) = connect(Url::parse("ws://localhost:8080").unwrap().as_str())
        .expect("Can't connect");
    
    println!("Connected: {}", response.status());

    // Send a message
    socket.send(Message::Text("client".into())).unwrap();

    loop {
        // Clear the screen with a specific color
        clear_background(LIGHTGRAY);

        // Draw a red circle in the center of the screen
        draw_circle(screen_width() / 2.0, screen_height() / 2.0, 50.0, RED);

        // Draw some text
        draw_text("Hello world!", 20.0, 20.0, 30.0, DARKGRAY);

        // Wait for the next frame
        next_frame().await;
    }
}
