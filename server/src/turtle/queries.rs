use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::turtle::{Direction, FuelLevel, Slot};

/// This holds all the turtle queries, actions that don't change world state and cannot fail in-game
pub trait TurtleQuery: Serialize {
    // This is the return type from the turtle
    type Response: DeserializeOwned;
    const ACTION: &'static str;
    
    // How this query represents itself to the turtle
    fn to_payload(&self) -> Value {
        let mut value = serde_json::to_value(self).unwrap();
        
        if let Some(map) = value.as_object_mut() {
            // For structs with fields (like GetItemCount), inject the tag
            map.insert("type".to_string(), Value::String(Self::ACTION.to_string()));
        } else {
            // For empty structs (like Detect), replace the Unit/Null with a new object
            value = json!({ "type": Self::ACTION.to_string() })
        }
        
        value
    }
}

/// All queries
#[derive(Serialize)]
pub struct Detect;
impl TurtleQuery for Detect { const ACTION: &'static str = "Detect"; type Response = Option<bool>; }

#[derive(Serialize)]
pub struct DetectUp;
impl TurtleQuery for DetectUp { const ACTION: &'static str = "DetectUp"; type Response = Option<bool>; }

#[derive(Serialize)]
pub struct DetectDown;
impl TurtleQuery for DetectDown { const ACTION: &'static str = "DetectDown"; type Response = Option<bool>; }

#[derive(Serialize)]
pub struct Inspect;
impl TurtleQuery for Inspect { const ACTION: &'static str = "Inspect"; type Response = Option<Value>; }

#[derive(Serialize)]
pub struct InspectUp;
impl TurtleQuery for InspectUp { const ACTION: &'static str = "InspectUp"; type Response = Option<Value>; }

#[derive(Serialize)]
pub struct InspectDown;
impl TurtleQuery for InspectDown { const ACTION: &'static str = "InspectDown"; type Response = Option<Value>; }

#[derive(Serialize)]
pub struct GetSelectedSlot;
impl TurtleQuery for GetSelectedSlot { const ACTION: &'static str = "GetSelectedSlot"; type Response = Slot; }

#[derive(Serialize)]
pub struct GetItemCount { pub slot: Option<Slot> }
impl TurtleQuery for GetItemCount { const ACTION: &'static str = "GetItemCount"; type Response = u8; }

#[derive(Serialize)]
pub struct GetItemSpace { pub slot: Option<Slot> }
impl TurtleQuery for GetItemSpace { const ACTION: &'static str = "GetItemSpace"; type Response = u8; }

#[derive(Serialize)]
pub struct GetItemDetail { pub slot: Option<Slot>, pub detailed: Option<bool> }
impl TurtleQuery for GetItemDetail { const ACTION: &'static str = "GetItemDetail"; type Response = Option<Value>; }

#[derive(Serialize)]
pub struct Compare;
impl TurtleQuery for Compare { const ACTION: &'static str = "Compare"; type Response = bool; }

#[derive(Serialize)]
pub struct CompareUp;
impl TurtleQuery for CompareUp { const ACTION: &'static str = "CompareUp"; type Response = bool; }

#[derive(Serialize)]
pub struct CompareDown;
impl TurtleQuery for CompareDown { const ACTION: &'static str = "CompareDown"; type Response = bool; }

#[derive(Serialize)]
pub struct CompareTo { pub slot: Slot }
impl TurtleQuery for CompareTo { const ACTION: &'static str = "CompareTo"; type Response = bool; }

#[derive(Serialize)]
pub struct GetFuelLevel;
impl TurtleQuery for GetFuelLevel { const ACTION: &'static str = "GetFuelLevel"; type Response = FuelLevel; }

#[derive(Serialize)]
pub struct GetFuelLimit;
impl TurtleQuery for GetFuelLimit { const ACTION: &'static str = "GetFuelLimit"; type Response = FuelLevel; }

#[derive(Serialize)]
pub struct GetEquippedLeft;
impl TurtleQuery for GetEquippedLeft { const ACTION: &'static str = "GetEquippedLeft"; type Response = Option<Value>; }

#[derive(Serialize)]
pub struct GetEquippedRight;
impl TurtleQuery for GetEquippedRight { const ACTION: &'static str = "GetEquippedRight"; type Response = Option<Value>; }

#[derive(Serialize)]
pub struct TurtleInit { pub version: u64, pub script: String }
impl TurtleQuery for TurtleInit { const ACTION: &'static str = "TurtleInit"; type Response = (i64, i64, i64, Direction); }