use std::collections::HashMap;

use macroquad::prelude::*;
use macroquad::ui::{root_ui, hash};
use shared::blocks::BlockNotification;
use shared::op_codes::ClientOpCode;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use url::Url;

#[macroquad::main("vTurtle Client")]
async fn main() {
    // Connect to a WebSocket server
    let (mut socket, response) = connect(Url::parse("ws://localhost:8080").unwrap().as_str())
        .expect("Can't connect");
    
    println!("Connected: {}", response.status());
    
    match socket.get_mut() {
        MaybeTlsStream::Plain(s) => s.set_nonblocking(true),
        _ => Ok(()),
    }.expect("Failed to set non-blocking");

    // Send a client qualifier and then ask for all blocks
    socket.send(Message::Text("client".into())).unwrap();
    socket.send(Message::Text(serde_json::to_string(&ClientOpCode::GetBlocks).unwrap().into())).unwrap();

    let mut blocks: HashMap<(i64, i64, i64), String> = HashMap::new();

    let mut camera_pos = vec3(10.0, 10.0, 10.0);
    let mut camera_pitch = 0.0f32;
    let mut camera_yaw = 0.0f32;
    let camera_speed = 0.1f32;
    let mouse_sensitivity = 0.03f32;

    loop {
        // Camera controls
        set_cursor_grab(true);
        show_mouse(false);
        let mouse = mouse_delta_position();
        camera_yaw += mouse.x * mouse_sensitivity;
        camera_pitch -= mouse.y * mouse_sensitivity;
        camera_pitch = camera_pitch.clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);

        let forward = vec3(camera_yaw.cos() * camera_pitch.cos(), camera_pitch.sin(), camera_yaw.sin() * camera_pitch.cos());
        let right = vec3((camera_yaw - std::f32::consts::PI / 2.0).cos(), 0.0, (camera_yaw - std::f32::consts::PI / 2.0).sin()).normalize();
        let up = vec3(0.0, 1.0, 0.0);

        if is_key_down(KeyCode::W) {
            camera_pos += forward * camera_speed;
        }
        if is_key_down(KeyCode::S) {
            camera_pos -= forward * camera_speed;
        }
        if is_key_down(KeyCode::D) {
            camera_pos += right * camera_speed;
        }
        if is_key_down(KeyCode::A) {
            camera_pos -= right * camera_speed;
        }
        if is_key_down(KeyCode::Space) {
            camera_pos -= up * camera_speed;
        }
        if is_key_down(KeyCode::LeftControl) {
            camera_pos += up * camera_speed;
        }

        let camera = Camera3D {
            position: camera_pos,
            target: camera_pos + forward,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 80.0,
            ..Default::default()
        };

        // Consume messages from the server
        if let Ok(msg) = socket.read() {
            match msg {
                Message::Text(text) => {
                    let notif: BlockNotification = serde_json::from_str(&text).unwrap();

                    match notif {
                        BlockNotification::Update(block) => {
                            println!("Block update: {}, {}, {} -> {}", block.x, block.y, block.z, block.block_type);
                            blocks.insert((block.x, block.y, block.z), block.block_type);
                        },
                        BlockNotification::Remove(x, y, z) => {
                            println!("Block removed: {}, {}, {}", x, y, z);
                            blocks.remove(&(x, y, z));
                        },
                    }
                },
                Message::Binary(_) => println!("Received binary data"),
                Message::Ping(_) | Message::Pong(_) => println!("Received ping/pong"),
                Message::Frame(_) => {},
                Message::Close(_) => {
                    println!("Connection closed by server");
                    break;
                }
            }
        }

        // Clear the screen with a specific color
        clear_background(LIGHTGRAY);
        set_camera(&camera);

        for ((x, y, z), _block_type) in blocks.iter() {
            let position = vec3(*x as f32, *y as f32, *z as f32);
            draw_cube_wires(position, Vec3::new(1.0, 1.0, 1.0), DARKGRAY);
        }

        draw_grid(20, 1.0, LIGHTGRAY, GRAY);

        set_default_camera();
        draw_text("vTurtle Client", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await;
    }
}
