use gamebox::{
    engines::game::map::{Direction, ElemColor, PhaseOffset},
    Vec3,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed, LengthDelimitedCodec};

pub type FramedTcpStream = Framed<TcpStream, LengthDelimitedCodec>;
pub type Hash = blake3::Hash;
pub type NotNan<T> = ordered_float::NotNan<T>;

pub fn framed_tcp_stream(tcp_stream: TcpStream) -> FramedTcpStream {
    LengthDelimitedCodec::new().framed(tcp_stream)
}

pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(&value)
}

pub fn deserialize<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}

pub fn hash(bytes: &[u8]) -> Hash {
    blake3::Hasher::new().update(bytes).finalize()
}

#[derive(Serialize, Deserialize)]
pub enum Mood {
    Day,
    Sunset,
    Night,
    Sunrise,
}

#[derive(Serialize, Deserialize)]
pub struct MapParamsDesc {
    pub mood: Mood,
}

#[derive(Serialize, Deserialize)]
pub struct MapDesc {
    pub custom_blocks: Vec<CustomBlockDesc>,
    pub custom_items: Vec<CustomItemDesc>,
    pub blocks: Vec<BlockDesc>,
    pub ghost_blocks: Vec<GhostBlockDesc>,
    pub free_blocks: Vec<FreeBlockDesc>,
    pub items: Vec<ItemDesc>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomBlockDesc {
    pub bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomItemDesc {
    pub bytes: Vec<u8>,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockDesc {
    pub block_info_id: ModelId,
    pub coord: Vec3<u8>,
    pub dir: Direction,
    pub is_air_variant: bool,
    pub elem_color: ElemColor,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GhostBlockDesc {
    pub block_info_id: ModelId,
    pub coord: Vec3<u8>,
    pub dir: Direction,
    pub is_air_variant: bool,
    pub elem_color: ElemColor,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FreeBlockDesc {
    pub block_info_id: ModelId,
    pub position: Vec3<NotNan<f32>>,
    pub yaw: NotNan<f32>,
    pub pitch: NotNan<f32>,
    pub roll: NotNan<f32>,
    pub elem_color: ElemColor,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemDesc {
    pub item_model_id: ModelId,
    pub position: Vec3<NotNan<f32>>,
    pub yaw: NotNan<f32>,
    pub pitch: NotNan<f32>,
    pub roll: NotNan<f32>,
    pub pivot_position: Vec3<NotNan<f32>>,
    pub elem_color: ElemColor,
    pub anim_offset: PhaseOffset,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelId {
    Game { id: String },
    Custom { hash: Hash },
}
