Net::Socket@ g_socket;

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

    dictionary blockInfos;
    LoadBlockInfos(blockInfos, "GameCtnBlockInfoClassic");
    LoadBlockInfos(blockInfos, "GameCtnBlockInfoClassic/Deprecated");
    LoadBlockInfos(blockInfos, "GameCtnBlockInfoPillar");

    dictionary itemModels;

    auto fids = Fids::GetGameFolder("GameData/Stadium/Items");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto itemModel = cast<CGameItemModel>(Fids::Preload(fid));
        itemModels[itemModel.IdName] = @itemModel;
    }

   // ComputeCustomObjectHashes(Fids::GetUserFolder("Blocks"));
    // ComputeCustomObjectHashes(Fids::GetUserFolder("Items"));

    auto editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

    @g_socket = Net::Socket();
    g_socket.Connect("127.0.0.1", 8369);

    while (!g_socket.CanWrite()) {
        yield();
    }

    auto placeBlockHook = Dev::Hook(pfPlaceBlock, 0, "OnPlaceBlock");
    auto removeBlockHook = Dev::Hook(pfRemoveBlock, 0, "OnRemoveBlock");
    auto placeItemHook = Dev::Hook(pfPlaceItem, 0, "OnPlaceItem");
    auto removeItemHook = Dev::Hook(pfRemoveItem, 0, "OnRemoveItem");

    while (true) {
        while (!g_socket.CanRead()) {
            yield();
        }

        auto available = g_socket.Available();

        if (available == 0) {
            break;
        }

        auto length = g_socket.ReadUint32();
        auto json = g_socket.ReadRaw(length);

        auto commandValue = Json::Parse(json);

        if (commandValue.HasKey("PlaceBlock")) {
            auto blockValue = commandValue["PlaceBlock"];
            auto modelValue = blockValue["model"];
            string model_id = modelValue["Id"];
            auto coordValue = blockValue["coord"];
            uint8 x = coordValue["x"];
            uint8 y = coordValue["y"];
            uint8 z = coordValue["z"];
            auto dir = ParseDir(blockValue["dir"]);
            bool isGround = blockValue["is_ground"];
            bool isGhost = blockValue["is_ghost"];
            auto color = ParseColor(blockValue["color"]);

            PlaceBlock(fnPlaceBlock, pfPlaceBlock, editor, blockInfo, x, y, z, dir, isGround, isGhost, color);
        } else if (commandValue.HasKey("RemoveBlock")) {
            auto blockValue = commandValue["RemoveBlock"];
        } else if (commandValue.HasKey("PlaceFreeBlock")) {
            auto freeBlockValue = commandValue["PlaceFreeBlock"];
        } else if (commandValue.HasKey("RemoveFreeBlock")) {
            auto freeBlockValue = commandValue["RemoveFreeBlock"];
        } else if (commandValue.HasKey("PlaceItem")) {
            auto itemValue = commandValue["PlaceItem"];
        } else if (commandValue.HasKey("RemoveItem")) {
            auto itemValue = commandValue["RemoveItem"];
        }

        print(json);
    }

    Dev::Unhook(placeBlockHook);
    Dev::Unhook(removeBlockHook);
    Dev::Unhook(placeItemHook);
    Dev::Unhook(removeItemHook);

    g_socket.Close();
}

void OnDestroyed() {

}

void OnPlaceBlock(CGameCtnBlockInfo@ rdx, uint64 r9, uint64 r11) {
    uint64 rsp;

    if (r9 == r11 - 24) {
        rsp = r9 - 168;
    } else {
        rsp = r9 - 184;
    }

    auto blockInfo = rdx;
    auto coord = r9;
    auto dir = CGameEditorPluginMap::ECardinalDirections(Dev::ReadUInt32(rsp + 40));
    auto color = CGameEditorPluginMap::EMapElemColor(Dev::ReadUInt8(rsp + 48));
    auto isGhost = Dev::ReadUInt32(rsp + 80) != 0;
    auto isGround = Dev::ReadUInt32(rsp + 104) != 0;
    auto isFree = Dev::ReadUInt32(rsp + 136) != 0;
    auto transform = Dev::ReadUInt32(rsp + 144);

    auto blockValue = Json::Object();

    auto modelValue = Json::Object();
    auto article = cast<CGameCtnArticle>(blockInfo.ArticlePtr);

    if (article !is null && article.BlockItem_ItemModelArticle is null) {
        modelValue["Id"] = blockInfo.IdName;
    } else {

    }

    blockValue["model"] = modelValue;

    auto commandValue = Json::Object();

    if (!isFree) {
        auto coordValue = Json::Object();
        coordValue["x"] = Dev::ReadUInt32(coord);
        coordValue["y"] = Dev::ReadUInt32(coord + 4);
        coordValue["z"] = Dev::ReadUInt32(coord + 8);

        blockValue["coord"] = coordValue;
        blockValue["dir"] = SerializeDir(dir);
        blockValue["is_ground"] = isGround;
        blockValue["variant_index"] = 0;
        blockValue["is_ghost"] = isGhost;
        blockValue["color"] = SerializeColor(color);

        commandValue["PlaceBlock"] = blockValue;
    } else {
        auto posValue = Json::Object();
        posValue["x"] = Dev::ReadFloat(transform);
        posValue["y"] = Dev::ReadFloat(transform + 4);
        posValue["z"] = Dev::ReadFloat(transform + 8);

        blockValue["pos"] = posValue;
        blockValue["yaw"] = Dev::ReadFloat(transform + 12);
        blockValue["pitch"] = Dev::ReadFloat(transform + 16);
        blockValue["roll"] = Dev::ReadFloat(transform + 20);
        blockValue["color"] = SerializeColor(color);

        commandValue["PlaceFreeBlock"] = blockValue;
    }

    auto json = Json::Write(commandValue);

    g_socket.Write(uint(json.Length));
    g_socket.WriteRaw(json);
}

void OnRemoveBlock(CGameCtnBlock@ rdx) {
    auto block = rdx;

    auto blockValue = Json::Object();

    auto commandValue = Json::Object();
    commandValue["PlaceBlock"] = blockValue;

    auto json = Json::Write(commandValue);

    g_socket.Write(uint(json.Length));
    g_socket.WriteRaw(json);
}

void OnPlaceItem(CGameItemModel@ rdx, uint64 r8) {
    auto itemModel = rdx;
    auto itemParams = r8;

    auto itemValue = Json::Object();
    auto modelValue = Json::Object();

    if (itemModel.EntityModelEdition is null) {
        modelValue["Id"] = itemModel.IdName;
    } else {

    }

    itemValue["model"] = modelValue;

    auto posValue = Json::Object();
    posValue["x"] = Dev::ReadFloat(itemParams + 28);
    posValue["y"] = Dev::ReadFloat(itemParams + 32);
    posValue["z"] = Dev::ReadFloat(itemParams + 36);

    itemValue["pos"] = posValue;
    itemValue["yaw"] = Dev::ReadFloat(itemParams + 12);
    itemValue["pitch"] = Dev::ReadFloat(itemParams + 16);
    itemValue["roll"] = Dev::ReadFloat(itemParams + 20);

    auto pivotPosValue = Json::Object();
    pivotPosValue["x"] = Dev::ReadFloat(itemParams + 76);
    pivotPosValue["y"] = Dev::ReadFloat(itemParams + 80);
    pivotPosValue["z"] = Dev::ReadFloat(itemParams + 84);

    itemValue["pivot_pos"] = pivotPosValue;
    itemValue["color"] = SerializeColor(CGameEditorPluginMap::EMapElemColor(Dev::ReadUInt8(itemParams + 152)));
    itemValue["anim_offset"] = SerializePhaseOffset(CGameEditorPluginMap::EPhaseOffset(Dev::ReadUInt8(itemParams + 153)));

    auto commandValue = Json::Object();
    commandValue["PlaceItem"] = itemValue;

    auto json = Json::Write(commandValue);

    g_socket.Write(uint(json.Length));
    g_socket.WriteRaw(json);
}

void OnRemoveItem(CGameCtnAnchoredObject@ rdx) {
    auto item = rdx;

    auto itemValue = Json::Object();
    auto modelValue = Json::Object();

    if (true) {
        modelValue["Id"] = item.ItemModel.IdName;
    } else {

    }

    itemValue["model"] = modelValue;

    auto posValue = Json::Object();
    posValue["x"] = item.AbsolutePositionInMap.x;
    posValue["y"] = item.AbsolutePositionInMap.y;
    posValue["z"] = item.AbsolutePositionInMap.z;

    itemValue["pos"] = posValue;
    itemValue["yaw"] = item.Yaw;
    itemValue["roll"] = item.Pitch;
    itemValue["pitch"] = item.Roll;

    auto pivotPosValue = Json::Object();
    
    itemValue["color"] = SerializeColor(item.MapElemColor);
    itemValue["anim_offset"] = SerializePhaseOffset(item.AnimPhaseOffset);

    auto commandValue = Json::Object();
    commandValue["RemoveItem"] = itemValue;

    auto json = Json::Write(commandValue);

    g_socket.Write(uint(json.Length));
    g_socket.WriteRaw(json);
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

void RemoveBlock(Import::Function@ fnRemoveBlock, uint64 pfRemoveBlock, CGameCtnEditorCommon@ editor, CGameCtnBlock@ block) {
    fnRemoveBlock.Call(pfRemoveBlock, editor, block);
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

void RemoveItem(Import::Function@ fnRemoveItem, uint64 pfRemoveItem, CGameCtnEditorCommon@ editor, CGameCtnAnchoredObject@ item) {
    fnRemoveItem.Call(pfRemoveItem, editor, item);
}

CGameCtnBlockInfo@ LoadBlockInfo(Import::Function@ fnLoadBlockInfo, uint64 pfLoadBlockInfo, CGameBlockItem@ blockItem) {
    return cast<CGameCtnBlockInfo>(fnLoadBlockInfo.CallNod(pfLoadBlockInfo, blockItem));
}

void LoadBlockInfos(dictionary@ blockInfos, const string&in folder) {
    auto fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/" + folder);

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfos[blockInfo.IdName] = @blockInfo;
    }
}

void ComputeCustomObjectHashes(CSystemFids@ fids) {
    for (uint i = 0; i < fids.Trees.Length; i++) {
        ComputeCustomObjectHashes(fids.Trees[i]);
    }

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];

        IO::File file(fid.FullFileName, IO::FileMode::Read);
        auto bytes = file.ReadToEnd();
        file.Close();

        auto hash = Crypto::Sha256(bytes);
    }
}

string SerializeDir(CGameEditorPluginMap::ECardinalDirections dir) {
    if (dir == CGameEditorPluginMap::ECardinalDirections::North) {
        return "North";
    } else if (dir == CGameEditorPluginMap::ECardinalDirections::East) {
        return "East";
    } else if (dir == CGameEditorPluginMap::ECardinalDirections::South) {
        return "South";
    } else if (dir == CGameEditorPluginMap::ECardinalDirections::West) {
        return "West";
    }

    return "";
}

string SerializeColor(CGameEditorPluginMap::EMapElemColor color) {
    if (color == CGameEditorPluginMap::EMapElemColor::Default) {
        return "Default";
    } else if (color == CGameEditorPluginMap::EMapElemColor::White) {
        return "White";
    } else if (color == CGameEditorPluginMap::EMapElemColor::Green) {
        return "Green";
    } else if (color == CGameEditorPluginMap::EMapElemColor::Blue) {
        return "Blue";
    } else if (color == CGameEditorPluginMap::EMapElemColor::Red) {
        return "Red";
    } else if (color == CGameEditorPluginMap::EMapElemColor::Black) {
        return "Black";
    }

    return "";
}

string SerializePhaseOffset(CGameEditorPluginMap::EPhaseOffset phaseOffset) {
    if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::None) {
        return "None";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::One8th) {
        return "One8th";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::Two8th) {
        return "Two8th";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::Three8th) {
        return "Three8th";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::Four8th) {
        return "Four8th";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::Five8th) {
        return "Five8th";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::Six8th) {
        return "Six8th";
    } else if (phaseOffset == CGameEditorPluginMap::EPhaseOffset::Seven8th) {
        return "Seven8th";
    }

    return "";
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
