use gamebox::{
    engines::game::map::{Direction, ElemColor, PhaseOffset, YawPitchRoll},
    Vec3,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed, LengthDelimitedCodec};

pub type FramedTcpStream = Framed<TcpStream, LengthDelimitedCodec>;
pub type Hash = blake3::Hash;

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

#[derive(Serialize, Deserialize)]
pub struct BlockDesc {
    pub block_info_id: ModelId,
    pub coord: Vec3<u8>,
    pub dir: Direction,
    pub elem_color: ElemColor,
}

#[derive(Serialize, Deserialize)]
pub struct GhostBlockDesc {
    pub block_info_id: ModelId,
    pub coord: Vec3<u8>,
    pub dir: Direction,
    pub elem_color: ElemColor,
}

#[derive(Serialize, Deserialize)]
pub struct FreeBlockDesc {
    pub block_info_id: ModelId,
    pub pos: Vec3<f32>,
    pub rotation: YawPitchRoll,
    pub elem_color: ElemColor,
}

#[derive(Serialize, Deserialize)]
pub struct ItemDesc {
    pub item_model_id: ModelId,
    pub pos: Vec3<f32>,
    pub pivot_pos: Vec3<f32>,
    pub rotation: YawPitchRoll,
    pub elem_color: ElemColor,
    pub anim_offset: PhaseOffset,
}

#[derive(Serialize, Deserialize)]
pub enum ModelId {
    Game { id: String },
    Custom { hash: Hash },
}
