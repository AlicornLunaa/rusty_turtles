#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TaskAction {
    Place{ x: i64, y: i64, z: i64, block: String },
    Break{ x: i64, y: i64, z: i64 }
}