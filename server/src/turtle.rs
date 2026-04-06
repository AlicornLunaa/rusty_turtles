pub struct Turtle {
    pub responder: simple_websockets::Responder,
    client_id: u64, // The unique identifier for the turtle agent
    x: i64,
    y: i64,
    z: i64,
    rotation: i8,
}

impl Turtle {
    pub fn new(responder: simple_websockets::Responder, client_id: u64) -> Self {
        Self { responder, client_id, x: 0, y: 0, z: 0, rotation: 0 }
    }

    pub fn update_spatial(&mut self, x: i64, y: i64, z: i64, rotation: i8) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.rotation = rotation;
    }

    pub fn get_client_id(&self) -> u64 {
        self.client_id
    }

    pub fn get_position(&self) -> (i64, i64, i64) {
        (self.x, self.y, self.z)
    }

    pub fn get_rotation(&self) -> i8 {
        self.rotation
    }
}