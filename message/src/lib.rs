use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    PlaceBlock,
    RemoveBlock,
    PlaceItem,
    RemoveItem,
}
