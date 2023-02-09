void Main() {
    auto pfPlaceBlock = Dev::FindPattern("48 89 5c 24 10 48 89 74 24 20 4c 89 44 24 18 55 57 41 55");
    auto pfRemoveBlock = Dev::FindPattern("48 89 5c 24 08 48 89 6c 24 10 48 89 74 24 18 57 48 83 ec 40 83 7c");
    auto pfPlaceItem = Dev::FindPattern("48 89 5c 24 18 55 56 57 48 83 ec 40 49");
    auto pfRemoveItem = Dev::FindPattern("48 89 5c 24 08 57 48 83 ec 30 48 8b fa 48 8b d9 48 85 d2 0f");
    auto pfLoadBlockInfo = Dev::FindPattern("48 83 ec 28 e8 a7 ff ff ff 48 85 c0 75 05");

    auto library = Import::GetLibrary("SyncEdit.dll");
    auto fnPlaceBlock = library.GetFunction("PlaceBlock");
    auto fnPlaceFreeBlock = library.GetFunction("PlaceFreeBlock");
    auto fnRemoveBlock = library.GetFunction("RemoveBlock");
    auto fnPlaceItem = library.GetFunction("PlaceItem");
    auto fnRemoveItem = library.GetFunction("RemoveItem");
    auto fnLoadBlockInfo = library.GetFunction("LoadBlockInfo");

    auto editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

    dictionary blockInfos;

    auto fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/GameCtnBlockInfoClassic");
    
    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfos[blockInfo.IdName] = @blockInfo;
    }

    @fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/GameCtnBlockInfoClassic/Deprecated");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfos[blockInfo.IdName] = @blockInfo;
    }

    @fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/GameCtnBlockInfoPillar");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfos[blockInfo.IdName] = @blockInfo;
    }

    dictionary itemModels;

    @fids = Fids::GetGameFolder("GameData/Stadium/Items");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto itemModel = cast<CGameItemModel>(Fids::Preload(fid));
        itemModels[itemModel.IdName] = @itemModel;
    }

    Net::Socket socket;
    socket.Connect("127.0.0.1", 8369);

    while (!socket.CanWrite()) {
        yield();
    }

    while (!socket.CanRead()) {
        yield();
    }

    auto startTime = Time::Now;

    auto available = socket.Available();
    socket.ReadUint32();
    auto json = socket.ReadRaw(available - 4);
    auto mapValue = Json::Parse(json);

    dictionary embeddedBlocks;

    auto embeddedBlocksValue = mapValue["embedded_blocks"];

    for (uint i = 0; i < embeddedBlocksValue.GetKeys().Length; i++) {
        auto hash = embeddedBlocksValue.GetKeys()[i];
        string base64 = embeddedBlocksValue.Get(hash);
        MemoryBuffer buffer;
        buffer.WriteFromBase64(base64);
        IO::File file(IO::FromUserGameFolder("temp.Block.Gbx"), IO::FileMode::Write);
        file.Write(buffer);
        file.Close();
        auto fid = Fids::GetUser("temp.Block.Gbx");
        auto nod = Fids::Preload(fid);
        IO::Delete("temp.Block.Gbx");
        auto itemModel = cast<CGameItemModel>(nod);
        auto blockItem = cast<CGameBlockItem>(itemModel.EntityModelEdition);
        auto blockInfo = LoadBlockInfo(fnLoadBlockInfo, pfLoadBlockInfo, blockItem);
        embeddedBlocks[hash] = @blockInfo;
    }

    dictionary embeddedItems;

    auto embeddedItemsValue = mapValue["embedded_items"];

    for (uint i = 0; i < embeddedItemsValue.GetKeys().Length; i++) {
        auto hash = embeddedItemsValue.GetKeys()[i];
        string base64 = embeddedItemsValue.Get(hash);
        MemoryBuffer buffer;
        buffer.WriteFromBase64(base64);
        IO::File file(IO::FromUserGameFolder("temp.Item.Gbx"), IO::FileMode::Write);
        file.Write(buffer);
        file.Close();
        auto fid = Fids::GetUser("temp.Item.Gbx");
        auto nod = Fids::Preload(fid);
        IO::Delete("temp.Item.Gbx");
        auto itemModel = cast<CGameItemModel>(nod);
        embeddedItems[hash] = @itemModel;
    }

    auto blocksValue = mapValue["blocks"];

    for (uint i = 0; i < blocksValue.Length; i++) {
        auto blockValue = blocksValue[i];

        auto modelvalue = blockValue["model"];
        CGameCtnBlockInfo@ model;

        if (modelvalue.HasKey("Id")) {
            string modelId = modelvalue["Id"];
            blockInfos.Get(modelId, @model);
        } else {
            string modelHash = modelvalue["Hash"];
            embeddedBlocks.Get(modelHash, @model);
        }
    
        auto coordValue = blockValue["coord"];
        uint8 x = coordValue["x"];
        uint8 y = coordValue["y"];
        uint8 z = coordValue["z"];

        auto dir = ParseDir(blockValue["dir"]);
        bool isGround = blockValue["is_ground"];
        bool isGhost = blockValue["is_ghost"];
        auto color = ParseColor(blockValue["color"]);

        PlaceBlock(fnPlaceBlock, pfPlaceBlock, editor, model, x, y, z, dir, isGround, isGhost, color);
    }

    auto freeBlocksValue = mapValue["free_blocks"];

    for (uint i = 0; i < freeBlocksValue.Length; i++) {
        auto freeBlockValue = freeBlocksValue[i];

        auto modelvalue = freeBlockValue["model"];
        CGameCtnBlockInfo@ model;

        if (modelvalue.HasKey("Id")) {
            string modelId = modelvalue["Id"];
            blockInfos.Get(modelId, @model);
        } else {
            string modelHash = modelvalue["Hash"];
            embeddedBlocks.Get(modelHash, @model);
        }
    
        auto posValue = freeBlockValue["pos"];
        float x = posValue["x"];
        float y = posValue["y"];
        float z = posValue["z"];

        float yaw = freeBlockValue["yaw"];
        float pitch = freeBlockValue["pitch"];
        float roll = freeBlockValue["roll"];
        auto color = ParseColor(freeBlockValue["color"]);

        PlaceFreeBlock(fnPlaceFreeBlock, pfPlaceBlock, editor, model, x, y, z, yaw, pitch, roll, color);
    }

    auto itemsValue = mapValue["items"];

    for (uint i = 0; i < itemsValue.Length; i++) {
        auto itemValue = itemsValue[i];

        auto modelvalue = itemValue["model"];
        CGameItemModel@ model;

        if (modelvalue.HasKey("Id")) {
            string modelId = modelvalue["Id"];
            itemModels.Get(modelId, @model);
        } else {
            string modelHash = modelvalue["Hash"];
            embeddedItems.Get(modelHash, @model);
        }

        auto posValue = itemValue["pos"];
        float x = posValue["x"];
        float y = posValue["y"];
        float z = posValue["z"];

        float yaw = itemValue["yaw"];
        float pitch = itemValue["pitch"];
        float roll = itemValue["roll"];

        auto pivotPosValue = itemValue["pos"];
        float pivotX = pivotPosValue["x"];
        float pivotY = pivotPosValue["y"];
        float pivotZ = pivotPosValue["z"];

        auto color = ParseColor(itemValue["color"]);
        auto animOffset = CGameEditorPluginMap::EPhaseOffset::None;

        PlaceItem(fnPlaceItem, pfPlaceItem, editor, model, x, y, z, yaw, pitch, roll, pivotX, pivotY, pivotZ, color, animOffset);
    }

    auto endTime = Time::Now;
    
    print("time: " + (endTime - startTime));
}

void PlaceBlock(
    Import::Function@ fnPlaceBlock, 
    uint64 pfPlaceBlock, 
    CGameCtnEditorCommon@ editor, 
    CGameCtnBlockInfo@ blockInfo,
    uint8 x,
    uint8 y,
    uint8 z,
    CGameEditorPluginMap::ECardinalDirections dir,
    bool isGround,
    bool isGhost,
    CGameEditorPluginMap::EMapElemColor color
) {
    fnPlaceBlock.Call(pfPlaceBlock, editor, blockInfo, int3(x, y, z), uint32(dir), isGround, isGhost, uint8(color));
}

void PlaceFreeBlock(
    Import::Function@ fnPlaceBlock, 
    uint64 pfPlaceBlock, 
    CGameCtnEditorCommon@ editor, 
    CGameCtnBlockInfo@ blockInfo,
    float x,
    float y,
    float z,
    float yaw,
    float pitch, 
    float roll,
    CGameEditorPluginMap::EMapElemColor color
) {
    fnPlaceBlock.Call(pfPlaceBlock, editor, blockInfo, vec3(x, y, z), vec3(yaw, pitch, roll), uint8(color));
}

void PlaceItem(
    Import::Function@ fnPlaceItem, 
    uint64 pfPlaceItem, 
    CGameCtnEditorCommon@ editor, 
    CGameItemModel@ itemModel,
    float x,
    float y,
    float z,
    float yaw,
    float pitch, 
    float roll,
    float pivotX,
    float pivotY,
    float pivotZ,
    CGameEditorPluginMap::EMapElemColor color,
    CGameEditorPluginMap::EPhaseOffset animOffset
) {
    fnPlaceItem.Call(pfPlaceItem, editor, itemModel, vec3(x, y, z), vec3(yaw, pitch, roll), vec3(pivotX, pivotY, pivotZ), uint8(color), uint8(animOffset));
}

CGameCtnBlockInfo@ LoadBlockInfo(Import::Function@ fnLoadBlockInfo, uint64 pfLoadBlockInfo, CGameBlockItem@ blockItem) {
    return cast<CGameCtnBlockInfo>(fnLoadBlockInfo.CallNod(pfLoadBlockInfo, blockItem));
}

CGameEditorPluginMap::ECardinalDirections ParseDir(const string&in str) {
    if (str == "North") {
        return CGameEditorPluginMap::ECardinalDirections::North;
    } else if (str == "East") {
        return CGameEditorPluginMap::ECardinalDirections::East;
    } else if (str == "South") {
        return CGameEditorPluginMap::ECardinalDirections::South;
    } else if (str == "West") {
        return CGameEditorPluginMap::ECardinalDirections::West;
    } else {
        return CGameEditorPluginMap::ECardinalDirections::North;
    }
}

CGameEditorPluginMap::EMapElemColor ParseColor(const string&in str) {
    if (str == "Default") {
        return CGameEditorPluginMap::EMapElemColor::Default;
    } else if (str == "White") {
        return CGameEditorPluginMap::EMapElemColor::White;
    } else if (str == "Green") {
        return CGameEditorPluginMap::EMapElemColor::Green;
    } else if (str == "Blue") {
        return CGameEditorPluginMap::EMapElemColor::Blue;
    } else if (str == "Red") {
        return CGameEditorPluginMap::EMapElemColor::Red;
    } else if (str == "Black") {
        return CGameEditorPluginMap::EMapElemColor::Black;
    } else {
        return CGameEditorPluginMap::EMapElemColor::Default;
    }
}
