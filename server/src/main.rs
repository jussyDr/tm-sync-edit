use std::{
    collections::HashMap,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use futures_util::{SinkExt, TryStreamExt};
use gamebox::engines::game::map::BlockKind;
use log::LevelFilter;
use shared::{
    framed_tcp_stream, serialize, BlockDesc, FreeBlockDesc, GhostBlockDesc, ItemDesc, MapDesc,
    MapParamsDesc, Mood,
};
use tokio::{net::TcpListener, runtime, spawn, sync::Mutex};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init()?;

    let runtime = runtime::Builder::new_multi_thread().enable_io().build()?;

    runtime.block_on(async {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8369);

        let tcp_listerner = TcpListener::bind(&socket_addr).await?;

        log::info!("listening on {socket_addr}");

        let state = Arc::new(Mutex::new(State::new()));

        loop {
            let (tcp_stream, socket_addr) = tcp_listerner.accept().await?;

            let state = Arc::clone(&state);

            spawn(async move {
                log::info!("accepted connection to {socket_addr}");

                state.lock().await.clients.insert(socket_addr, ());

                let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

                let map_params_desc = MapParamsDesc { mood: Mood::Day };

                let frame = serialize(&map_params_desc).unwrap();
                framed_tcp_stream.send(frame.into()).await.unwrap();

                let map_desc = load_map();
                let frame = serialize(&map_desc).unwrap();
                framed_tcp_stream.send(frame.into()).await.unwrap();

                while framed_tcp_stream.try_next().await.unwrap().is_some() {}

                state.lock().await.clients.remove(&socket_addr);
            });
        }
    })
}

struct State {
    clients: HashMap<SocketAddr, ()>,
}

impl State {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}

pub fn load_map() -> MapDesc {
    let map: gamebox::Map =
        gamebox::read_file("C:\\Users\\Justin\\Projects\\tm-sync-edit\\G H O S T.Map.Gbx").unwrap();

    let mut blocks = vec![];
    let mut ghost_blocks = vec![];
    let mut free_blocks = vec![];

    for block in map.blocks() {
        match block.kind() {
            BlockKind::Normal(block_kind) => {
                if block_kind.is_ghost() {
                    ghost_blocks.push(GhostBlockDesc {
                        block_info_name: block.id().to_owned(),
                        coord: block_kind.coord(),
                        dir: block_kind.direction(),
                        elem_color: block.elem_color(),
                    })
                } else {
                    blocks.push(BlockDesc {
                        block_info_name: block.id().to_owned(),
                        coord: block_kind.coord(),
                        dir: block_kind.direction(),
                        elem_color: block.elem_color(),
                    })
                }
            }
            BlockKind::Free(block_kind) => free_blocks.push(FreeBlockDesc {
                block_info_name: block.id().to_owned(),
                position: block_kind.position().clone(),
                rotation: block_kind.rotation().clone(),
                elem_color: block.elem_color(),
            }),
        }
    }

    let mut items = vec![];

    for item in map.items() {
        items.push(ItemDesc {
            item_model_name: item.id().to_owned(),
            position: item.position().clone(),
            rotation: item.rotation().clone(),
            elem_color: item.elem_color(),
            anim_offset: item.animation_offset(),
        })
    }

    MapDesc {
        blocks,
        ghost_blocks,
        free_blocks,
        items,
    }
}
