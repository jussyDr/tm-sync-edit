const string c_title = Icons::Syncthing + " Sync Edit";

enum State {
    Disconnected,
    Connecting,
    Connected,
}

string g_status;

State g_state;
bool g_stopped;

Net::Socket@ g_socket;
Dev::HookInfo@ g_hookPlaceBlock;
Dev::HookInfo@ g_hookRemoveBlock;
Dev::HookInfo@ g_hookPlaceItem;
Dev::HookInfo@ g_hookRemoveItem;

bool g_disableHookCallbacks = false;

dictionary g_placedBlocks;
NodMultimap g_placedGhostBlocks;
NodMultimap g_placedFreeBlocks;
NodMultimap g_placedItems;
uint g_numNewBlocksPlaced;
uint g_numNewItemsPlaced;

void Main() {
    g_status = "Disconnected";
    g_state = State::Disconnected;
    g_stopped = false;
    g_numNewBlocksPlaced = 0;
    g_numNewItemsPlaced = 0;
}

void RenderInterface() {
    if (Setting_InterfaceVisible && UI::Begin(c_title)) {
        if (g_state == State::Disconnected) {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);
            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

            if (UI::Button("Join")) {
                startnew(MainLoop);
            }
        } else if (g_state == State::Connecting) {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                g_stopped = true;
            }
        } else if (g_state == State::Connected) {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Exit")) {
                g_stopped = true;
            }
        }

        UI::Text(g_status);

        UI::End();
    }
}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", Setting_InterfaceVisible)) {
        Setting_InterfaceVisible = !Setting_InterfaceVisible;
    }
}

void OnDisabled() {

}

void OnEnabled() {

}

void OnDestroyed() {
    if (g_hookPlaceBlock !is null) {
        Dev::Unhook(g_hookPlaceBlock);
        @g_hookPlaceBlock = null;
    }

    if (g_hookRemoveBlock !is null) {
        Dev::Unhook(g_hookRemoveBlock);
        @g_hookRemoveBlock = null;
    }

    if (g_hookPlaceItem !is null) {
        Dev::Unhook(g_hookPlaceItem);
        @g_hookPlaceItem = null;
    }

    if (g_hookRemoveItem !is null) {
        Dev::Unhook(g_hookRemoveItem);
        @g_hookRemoveItem = null;
    }

    if (g_socket !is null) {
        g_socket.Close();
        @g_socket = null;
    }

    g_state = State::Disconnected;
}

void OnPlaceBlock() {
    if (!g_disableHookCallbacks) {
        g_numNewBlocksPlaced++;
    }
}

void OnRemoveBlock(CGameCtnBlock@ rdx) {
    if (!g_disableHookCallbacks) {
        if (g_numNewBlocksPlaced > 0) {
            g_numNewBlocksPlaced--;
        } else {
            if (Block::IsFree(rdx)) {
                auto freeBlockValue = SerializeFreeBlock(rdx);
                auto freeBlockJson = Json::Write(freeBlockValue);
                SendCommand("RemoveFreeBlock", freeBlockJson);
                g_placedFreeBlocks.RemoveValue(freeBlockJson, rdx);
            } else {
                auto blockValue = SerializeBlock(rdx);
                auto blockJson = Json::Write(blockValue);

                if (Block::IsGhost(rdx)) {
                    SendCommand("RemoveGhostBlock", blockJson);
                    g_placedGhostBlocks.RemoveValue(blockJson, rdx);
                } else {
                    SendCommand("RemoveBlock", blockJson);
                    g_placedBlocks.Delete(blockJson);
                }
            }
        }
    }
}

void OnPlaceItem() {
    if (!g_disableHookCallbacks) {
        g_numNewItemsPlaced++;
    }
}

void OnRemoveItem(CGameCtnAnchoredObject@ rdx) {
    if (!g_disableHookCallbacks) {
        auto itemValue = SerializeItem(rdx);
        auto itemJson = Json::Write(itemValue);
        SendCommand("RemoveItem", itemJson);
        g_placedItems.RemoveValue(itemJson, rdx);
    }
}

void Error(const string&in message) {
    OnDestroyed();
    g_status = message;
}

void MainLoop() {
    g_state = State::Connecting;

    if (Setting_Port.Length == 0) {
        Error("Invalid port");
        return;
    }

    auto port = Text::ParseUInt(Setting_Port);
    
    if (port > 65535) {
        Error("Invalid port");
        return;
    }

    @g_socket = Net::Socket();

    if (!g_socket.Connect(Setting_Host, port)) {
        Error("Invalid host");
        return;
    }

    g_status = "Connecting";

    auto connectTime = Time::Now;

    while (!g_socket.CanWrite()) {
        yield();

        if (Time::Now >= connectTime + Setting_ConnectTimeout) {
            Error("Timed out");
            return;
        }

        if (g_stopped) {
            g_stopped = false;
            Error("Canceled");
            return;
        }
    }

    g_status = "Connected";
    g_state = State::Connected;

    dictionary officialBlockInfos;

    LoadOfficialBlockInfos(officialBlockInfos, "GameCtnBlockInfoClassic");
    LoadOfficialBlockInfos(officialBlockInfos, "GameCtnBlockInfoClassic/Deprecated");
    LoadOfficialBlockInfos(officialBlockInfos, "GameCtnBlockInfoPillar");

    dictionary officialItemModels;
    auto officialItemModelFids = Fids::GetGameFolder("GameData/Stadium/Items");

    for (uint i = 0; i < officialItemModelFids.Leaves.Length; i++) {
        auto fid = officialItemModelFids.Leaves[i];
        auto nod = Fids::Preload(fid);
        auto itemModel = cast<CGameItemModel>(nod);
        officialItemModels[itemModel.IdName] = @itemModel;
    }

    auto library = Import::GetLibrary("SyncEdit.dll");

    if (library is null) {
        Error("Failed to find library 'SyncEdit.dll'");
        return;
    }

    auto placeBlockFunc = library.GetFunction("PlaceBlock");

    if (placeBlockFunc is null) {
        Error("Failed to find library function 'PlaceBlock'");
        return;
    }

    auto placeFreeBlockFunc = library.GetFunction("PlaceFreeBlock");

    if (placeFreeBlockFunc is null) {
        Error("Failed to find library function 'PlaceFreeBlock'");
        return;
    }

    auto removeBlockFunc = library.GetFunction("RemoveBlock");

    if (removeBlockFunc is null) {
        Error("Failed to find library function 'RemoveBlock'");
        return;
    }

    auto placeItemFunc = library.GetFunction("PlaceItem");

    if (placeItemFunc is null) {
        Error("Failed to find library function 'PlaceItem'");
        return;
    }

    auto removeItemFunc = library.GetFunction("RemoveItem");

    if (removeItemFunc is null) {
        Error("Failed to find library function 'RemoveItem'");
        return;
    }

    auto pfPlaceBlock = Dev::FindPattern("48 89 5c 24 10 48 89 74 24 20 4c 89 44 24 18 55 57 41 55");

    if (pfPlaceBlock == 0) {
        Error("Failed to find function 'PlaceBlock'");
        return;
    }

    auto pfRemoveBlock = Dev::FindPattern("48 89 5c 24 08 48 89 6c 24 10 48 89 74 24 18 57 48 83 ec 40 83 7c");

    if (pfRemoveBlock == 0) {
        Error("Failed to find function 'RemoveBlock'");
        return;
    }

    auto pfPlaceItem = Dev::FindPattern("48 89 5c 24 18 55 56 57 48 83 ec 40 49");

    if (pfPlaceItem == 0) {
        Error("Failed to find function 'PlaceItem'");
        return;
    }

    auto pfRemoveItem = Dev::FindPattern("48 89 5c 24 08 57 48 83 ec 30 48 8b fa 48 8b d9 48 85 d2 0f");

    if (pfRemoveItem == 0) {
        Error("Failed to find function 'RemoveItem'");
        return;
    }

    @g_hookPlaceBlock = Dev::Hook(pfPlaceBlock, 0, "OnPlaceBlock");

    if (g_hookPlaceBlock is null) {
        Error("Failed to hook function 'PlaceBlock'");
        return;
    }
    
    @g_hookRemoveBlock = Dev::Hook(pfRemoveBlock, 0, "OnRemoveBlock");

    if (g_hookRemoveBlock is null) {
        Error("Failed to hook function 'RemoveBlock'");
        return;
    }

    @g_hookPlaceItem = Dev::Hook(pfPlaceItem, 0, "OnPlaceItem");

    if (g_hookPlaceItem is null) {
        Error("Failed to hook function 'PlaceItem'");
        return;
    }

    @g_hookRemoveItem = Dev::Hook(pfRemoveItem, 0, "OnRemoveItem");

    if (g_hookRemoveItem is null) {
        Error("Failed to hook function 'RemoveItem'");
        return;
    }

    while (true) {
        auto editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

        if (editor is null) {
            Error("Exited editor");
            return;
        }

        while (!g_socket.CanRead()) {
            yield();

            if (g_stopped) {
                g_stopped = false;
                Error("Exited");
                return;
            }

            @editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

            if (editor is null) {
                Error("Exited editor");
                return;
            }

            auto blocks = editor.Challenge.Blocks;

            while (g_numNewBlocksPlaced > 0) {
                auto block = blocks[blocks.Length - g_numNewBlocksPlaced];

                if (Block::IsFree(block)) {
                    auto freeBlockValue = SerializeFreeBlock(block);
                    auto freeBlockJson = Json::Write(freeBlockValue);
                    SendCommand("PlaceFreeBlock", freeBlockJson);
                    g_placedFreeBlocks.Insert(freeBlockJson, block);
                } else {
                    auto blockValue = SerializeBlock(block);
                    auto blockJson = Json::Write(blockValue);

                    if (Block::IsGhost(block)) {
                        SendCommand("PlaceGhostBlock", blockJson);
                        g_placedGhostBlocks.Insert(blockJson, block);
                    } else {
                        SendCommand("PlaceBlock", blockJson);
                        g_placedBlocks[blockJson] = @block;
                    }
                }

                g_numNewBlocksPlaced--;
            }

            auto items = editor.Challenge.AnchoredObjects;

            while (g_numNewItemsPlaced > 0) {
                auto item = items[items.Length - g_numNewItemsPlaced];
                auto itemValue = SerializeItem(item);
                auto itemJson = Json::Write(itemValue);
                SendCommand("PlaceItem", itemJson);
                g_placedItems.Insert(itemJson, item);

                g_numNewItemsPlaced--;
            }
        }

        uint available = g_socket.Available();

        if (available == 0) {
            Error("Server disconnected");
            return;
        }

        g_disableHookCallbacks = true;

        while (available > 0) {
            if (available < 4) {
                Error("Client implementation error");
                return;
            }

            uint frameLength = g_socket.ReadUint32();
            available -= 4;

            if (available < frameLength) {
                Error("Client implementation error");
                return;
            }

            auto json = g_socket.ReadRaw(frameLength);
            available -= frameLength;

            auto commandValue = Json::Parse(json);

            if (commandValue.HasKey("PlaceBlock")) {
                string blockJson = commandValue["PlaceBlock"];
                auto blockValue = Json::Parse(blockJson);
                auto modelValue = blockValue["model"];
                string modelId = modelValue["Id"];
                CGameCtnBlockInfo@ blockInfo;
                officialBlockInfos.Get(modelId, @blockInfo);
                auto coordValue = blockValue["coord"];
                uint8 x = coordValue["x"];
                uint8 y = coordValue["y"];
                uint8 z = coordValue["z"];
                auto dir = DeserializeDir(blockValue["dir"]);
                bool isGround = blockValue["is_ground"];
                uint8 variantIndex = blockValue["variant_index"];
                auto color = DeserializeColor(blockValue["color"]);

                auto block = Editor::PlaceBlock(editor, placeBlockFunc, pfPlaceBlock, blockInfo, x, y, z, dir, isGround, variantIndex, false, color);

                g_placedBlocks[blockJson] = @block;
            } else if (commandValue.HasKey("RemoveBlock")) {
                string blockJson = commandValue["RemoveBlock"];

                CGameCtnBlock@ block;
                g_placedBlocks.Get(blockJson, @block);

                if (block !is null) {
                    g_placedBlocks.Delete(blockJson);

                    Editor::RemoveBlock(editor, removeBlockFunc, pfRemoveBlock, block);
                }
            } else if (commandValue.HasKey("SetGhostBlockCount")) {
                auto value = commandValue["SetGhostBlockCount"];
                string blockJson = value["block_json"];
                uint newCount = value["count"];

                auto count = g_placedGhostBlocks.Exists(blockJson);

                if (count < newCount) {
                    auto blockValue = Json::Parse(blockJson);
                    auto modelValue = blockValue["model"];
                    string modelId = modelValue["Id"];
                    CGameCtnBlockInfo@ blockInfo;
                    officialBlockInfos.Get(modelId, @blockInfo);
                    auto coordValue = blockValue["coord"];
                    uint8 x = coordValue["x"];
                    uint8 y = coordValue["y"];
                    uint8 z = coordValue["z"];
                    auto dir = DeserializeDir(blockValue["dir"]);
                    bool isGround = blockValue["is_ground"];
                    uint8 variantIndex = blockValue["variant_index"];
                    auto color = DeserializeColor(blockValue["color"]);

                    do {
                        auto block = Editor::PlaceBlock(editor, placeBlockFunc, pfPlaceBlock, blockInfo, x, y, z, dir, isGround, variantIndex, true, color);

                        g_placedGhostBlocks.Insert(blockJson, block);

                        count++;
                    } while (count < newCount);
                } else {
                    while (count > newCount) {
                        auto nod = g_placedGhostBlocks.Remove(blockJson);
                        auto block = cast<CGameCtnBlock>(nod);

                        Editor::RemoveBlock(editor, removeBlockFunc, pfRemoveBlock, block);

                        count--;
                    }
                }
            } else if (commandValue.HasKey("SetFreeBlockCount")) {
                auto value = commandValue["SetFreeBlockCount"];
                string freeBlockJson = value["free_block_json"];
                uint newCount = value["count"];

                auto count = g_placedFreeBlocks.Exists(freeBlockJson);

                if (count < newCount) {
                    auto freeBlockValue = Json::Parse(freeBlockJson);
                    auto modelValue = freeBlockValue["model"];
                    string modelId = modelValue["Id"];
                    CGameCtnBlockInfo@ blockInfo;
                    officialBlockInfos.Get(modelId, @blockInfo);
                    auto pos = DeserializeVec3(freeBlockValue["pos"]);
                    float yaw = freeBlockValue["yaw"];
                    float pitch = freeBlockValue["pitch"];
                    float roll = freeBlockValue["roll"];
                    auto color = DeserializeColor(freeBlockValue["color"]);

                    do {
                        auto block = Editor::PlaceFreeBlock(editor, placeFreeBlockFunc, pfPlaceBlock, blockInfo, pos, yaw, pitch, roll, color);

                        g_placedFreeBlocks.Insert(freeBlockJson, block);

                        count++;
                    } while (count < newCount);
                } else {
                    while (count > newCount) {
                        auto nod = g_placedFreeBlocks.Remove(freeBlockJson);
                        auto block = cast<CGameCtnBlock>(nod);

                        Editor::RemoveBlock(editor, removeBlockFunc, pfRemoveBlock, block);

                        count--;
                    }
                }
            } else if (commandValue.HasKey("SetItemCount")) {
                auto value = commandValue["SetItemCount"];
                string itemJson = value["item_json"];
                uint newCount = value["count"];

                auto count = g_placedItems.Exists(itemJson);

                if (count < newCount) {
                    auto itemValue = Json::Parse(itemJson);
                    auto modelValue = itemValue["model"];
                    string modelId = modelValue["Id"];
                    CGameItemModel@ itemModel;
                    officialItemModels.Get(modelId, @itemModel);
                    auto pos = DeserializeVec3(itemValue["pos"]);
                    float yaw = itemValue["yaw"];
                    float pitch = itemValue["pitch"];
                    float roll = itemValue["roll"];
                    auto pivotPos = DeserializeVec3(itemValue["pivot_pos"]);
                    uint8 variantIndex = itemValue["variant_index"];
                    auto color = DeserializeColor(itemValue["color"]);
                    auto animOffset = DeserializePhaseOffset(itemValue["anim_offset"]);

                    do {
                        auto item = Editor::PlaceItem(editor, placeItemFunc, pfPlaceItem, itemModel, pos, yaw, pitch, roll, pivotPos, variantIndex, color, animOffset);

                        g_placedItems.Insert(itemJson, item);

                        count++;
                    } while (count < newCount);
                } else {
                    while (count > newCount) {
                        auto nod = g_placedItems.Remove(itemJson);
                        auto item = cast<CGameCtnAnchoredObject>(nod);

                        Editor::RemoveItem(editor, removeItemFunc, pfRemoveItem, item);

                        count--;
                    }
                }
            }
        }

        g_disableHookCallbacks = false;
    }
}

void SendCommand(const string&in name, const Json::Value@ value) {
    auto commandValue = Json::Object();
    commandValue[name] = value;

    auto json = Json::Write(commandValue);
    g_socket.Write(json);
}

const Json::Value@ SerializeBlock(CGameCtnBlock@ block) {
    auto modelValue = Json::Object();
    modelValue["Id"] = block.BlockInfo.IdName;

    auto coordValue = Json::Object();
    coordValue["x"] = block.CoordX;
    coordValue["y"] = block.CoordY;
    coordValue["z"] = block.CoordZ;

    auto blockValue = Json::Object();
    blockValue["model"] = modelValue;
    blockValue["coord"] = coordValue;
    blockValue["dir"] = SerializeDir(block.Dir);
    blockValue["is_ground"] = block.IsGround;
    blockValue["variant_index"] = block.BlockInfoVariantIndex;
    blockValue["color"] = SerializeColor(CGameEditorPluginMap::EMapElemColor(block.MapElemColor));

    return blockValue;
}

const Json::Value@ SerializeFreeBlock(CGameCtnBlock@ block) {
    auto pBlock = Dev::ForceCast<uint64>(block).Get();
    
    auto modelValue = Json::Object();
    modelValue["Id"] = block.BlockInfo.IdName;

    auto blockValue = Json::Object();
    blockValue["model"] = modelValue;
    blockValue["pos"] = SerializeVec3(Dev::ReadVec3(pBlock + 108));
    blockValue["yaw"] = Dev::ReadFloat(pBlock + 120);
    blockValue["pitch"] = Dev::ReadFloat(pBlock + 124);
    blockValue["roll"] = Dev::ReadFloat(pBlock + 128);
    blockValue["color"] = SerializeColor(CGameEditorPluginMap::EMapElemColor(block.MapElemColor));

    return blockValue;
}

const Json::Value@ SerializeItem(CGameCtnAnchoredObject@ item) {
    auto pItem = Dev::ForceCast<uint64>(item).Get();

    auto modelValue = Json::Object();
    modelValue["Id"] = item.ItemModel.IdName;

    auto itemValue = Json::Object();
    itemValue["model"] = modelValue;
    itemValue["pos"] = SerializeVec3(item.AbsolutePositionInMap);
    itemValue["yaw"] = item.Yaw;
    itemValue["pitch"] = item.Pitch;
    itemValue["roll"] = item.Roll;
    itemValue["pivot_pos"] = SerializeVec3(Dev::ReadVec3(pItem + 116));
    itemValue["variant_index"] = item.IVariant;
    itemValue["color"] = SerializeColor(CGameEditorPluginMap::EMapElemColor(item.MapElemColor));
    itemValue["anim_offset"] = SerializePhaseOffset(CGameEditorPluginMap::EPhaseOffset(item.AnimPhaseOffset));

    return itemValue;
}

const Json::Value@ SerializeDir(CGameEditorPluginMap::ECardinalDirections dir) {
    if (dir == CGameEditorPluginMap::ECardinalDirections::North) {
        return "North";
    } else if (dir == CGameEditorPluginMap::ECardinalDirections::East) {
        return "East";
    } else if (dir == CGameEditorPluginMap::ECardinalDirections::South) {
        return "South";
    } else if (dir == CGameEditorPluginMap::ECardinalDirections::West) {
        return "West";
    }

    return null;
}

const Json::Value@ SerializeColor(CGameEditorPluginMap::EMapElemColor color) {
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

    return null;
}

const Json::Value@ SerializePhaseOffset(CGameEditorPluginMap::EPhaseOffset offset) {
    if (offset == CGameEditorPluginMap::EPhaseOffset::None) {
        return "None";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::One8th) {
        return "One8th";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::Two8th) {
        return "Two8th";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::Three8th) {
        return "Three8th";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::Four8th) {
        return "Four8th";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::Five8th) {
        return "Five8th";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::Six8th) {
        return "Six8th";
    } else if (offset == CGameEditorPluginMap::EPhaseOffset::Seven8th) {
        return "Seven8th";
    }

    return null;
}

const Json::Value@ SerializeVec3(const vec3&in vec) {
    auto vecValue = Json::Object();
    vecValue["x"] = vec.x;
    vecValue["y"] = vec.y;
    vecValue["z"] = vec.z;

    return vecValue;
}

CGameEditorPluginMap::ECardinalDirections DeserializeDir(const Json::Value@ dirValue) {
    if (dirValue == "North") {
        return CGameEditorPluginMap::ECardinalDirections::North;
    } else if (dirValue == "East") {
        return CGameEditorPluginMap::ECardinalDirections::East;
    } else if (dirValue == "South") {
        return CGameEditorPluginMap::ECardinalDirections::South;
    } else if (dirValue == "West") {
        return CGameEditorPluginMap::ECardinalDirections::West;
    }

    return CGameEditorPluginMap::ECardinalDirections::North;
}

CGameEditorPluginMap::EMapElemColor DeserializeColor(const Json::Value@ colorValue) {
    if (colorValue == "Default") {
        return CGameEditorPluginMap::EMapElemColor::Default;
    } else if (colorValue == "White") {
        return CGameEditorPluginMap::EMapElemColor::White;
    } else if (colorValue == "Green") {
        return CGameEditorPluginMap::EMapElemColor::Green;
    } else if (colorValue == "Blue") {
        return CGameEditorPluginMap::EMapElemColor::Blue;
    } else if (colorValue == "Red") {
        return CGameEditorPluginMap::EMapElemColor::Red;
    } else if (colorValue == "Black") {
        return CGameEditorPluginMap::EMapElemColor::Black;
    }

    return CGameEditorPluginMap::EMapElemColor::Default;
}

CGameEditorPluginMap::EPhaseOffset DeserializePhaseOffset(const Json::Value@ offsetValue) {
    if (offsetValue == "None") {
        return CGameEditorPluginMap::EPhaseOffset::None;
    } else if (offsetValue == "One8th") {
        return CGameEditorPluginMap::EPhaseOffset::One8th;
    } else if (offsetValue == "Two8th") {
        return CGameEditorPluginMap::EPhaseOffset::Two8th;
    } else if (offsetValue == "Three8th") {
        return CGameEditorPluginMap::EPhaseOffset::Three8th;
    } else if (offsetValue == "Four8th") {
        return CGameEditorPluginMap::EPhaseOffset::Four8th;
    } else if (offsetValue == "Five8th") {
        return CGameEditorPluginMap::EPhaseOffset::Five8th;
    } else if (offsetValue == "Six8th") {
        return CGameEditorPluginMap::EPhaseOffset::Six8th;
    } else if (offsetValue == "Seven8th") {
        return CGameEditorPluginMap::EPhaseOffset::Seven8th;
    }

    return CGameEditorPluginMap::EPhaseOffset::None;
}

vec3 DeserializeVec3(const Json::Value@ vecValue) {
    float x = vecValue["x"];
    float y = vecValue["y"];
    float z = vecValue["z"];

    return vec3(x, y, z);
}

void LoadOfficialBlockInfos(dictionary@ blockInfos, const string&in folder) {
    auto fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/" + folder);

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto nod = Fids::Preload(fid);
        auto blockInfo = cast<CGameCtnBlockInfo>(nod);
        blockInfos[blockInfo.IdName] = @blockInfo;
    }
}

namespace Block {
    uint8 Flags(CGameCtnBlock@ block) {
        auto pBlock = Dev::ForceCast<uint64>(block).Get();

        return Dev::ReadUInt8(pBlock + 135);
    } 

    bool IsGhost(CGameCtnBlock@ block) {
        return Flags(block) & 0x10 != 0;
    }

    bool IsFree(CGameCtnBlock@ block) {
        return Flags(block) & 0x20 != 0;
    }
}

namespace Editor {
    CGameCtnBlock@ PlaceBlock(
        CGameCtnEditorCommon@ editor, 
        Import::Function@ placeBlockFunc,
        uint64 pfPlaceBlock,
        CGameCtnBlockInfo@ blockInfo,
        uint8 x,
        uint8 y,
        uint8 z,
        CGameEditorPluginMap::ECardinalDirections dir,
        bool isGround,
        uint8 variantIndex,
        bool isGhost,
        CGameEditorPluginMap::EMapElemColor color
    ) {
        auto nod = placeBlockFunc.CallNod(
            pfPlaceBlock,
            editor,
            blockInfo,
            int3(x, y, z),
            uint(dir),
            isGround, 
            uint(variantIndex),
            isGhost,
            uint8(color)
        );

        return cast<CGameCtnBlock>(nod);
    }

    CGameCtnBlock@ PlaceFreeBlock(
        CGameCtnEditorCommon@ editor, 
        Import::Function@ placeFreeBlockFunc,
        uint64 pfPlaceBlock,
        CGameCtnBlockInfo@ blockInfo,
        vec3 pos,
        float yaw,
        float pitch,
        float roll,
        CGameEditorPluginMap::EMapElemColor color
    ) {
        auto nod = placeFreeBlockFunc.CallNod(
            pfPlaceBlock,
            editor,
            blockInfo,
            pos,
            vec3(yaw, pitch, roll),
            uint8(color)
        );

        return cast<CGameCtnBlock>(nod);
    }

    void RemoveBlock(
        CGameCtnEditorCommon@ editor, 
        Import::Function@ removeBlockFunc,
        uint64 pfRemoveBlock,
        CGameCtnBlock@ block
    ) {
        removeBlockFunc.Call(pfRemoveBlock, editor, block);
    }

    CGameCtnAnchoredObject@ PlaceItem(
        CGameCtnEditorCommon@ editor, 
        Import::Function@ placeItemFunc,
        uint64 pfPlaceItem,
        CGameItemModel@ itemModel,
        vec3 pos,
        float yaw,
        float pitch,
        float roll,
        vec3 pivotPos,
        uint8 variantIndex,
        CGameEditorPluginMap::EMapElemColor color,
        CGameEditorPluginMap::EPhaseOffset animOffset
    ) {
        auto nod = placeItemFunc.CallNod(
            pfPlaceItem,
            editor,
            itemModel,
            pos,
            vec3(yaw, pitch, roll),
            pivotPos,
            variantIndex,
            uint8(color),
            uint8(animOffset)
        );

        return cast<CGameCtnAnchoredObject>(nod);
    }

    void RemoveItem(
        CGameCtnEditorCommon@ editor, 
        Import::Function@ removeItemFunc,
        uint64 pfRemoveItem,
        CGameCtnAnchoredObject@ item
    ) {
        removeItemFunc.Call(pfRemoveItem, editor, item);
    }
}

class NodMultimap {
    dictionary m_map;

    uint Exists(const string&in key) {
        if (m_map.Exists(key)) {
            array<CMwNod@> list;
            m_map.Get(key, list);
            
            return list.Length;
        }

        return 0;
    }

    void Insert(const string&in key, CMwNod@ nod) {
        if (m_map.Exists(key)) {
            array<CMwNod@>@ list;
            m_map.Get(key, @list);
            list.InsertLast(nod);
        } else {
            array<CMwNod@>@ list = { nod };
            m_map[key] = @list;
        }
    }

    CMwNod@ Remove(const string&in key) {
        if (m_map.Exists(key)) {
            array<CMwNod@>@ list;
            m_map.Get(key, @list);

            auto nod = list[list.Length - 1];
            list.RemoveLast();
            
            return nod;
        }
            
        return null;
    }

    void RemoveValue(const string&in key, CMwNod@ nod) {
        if (m_map.Exists(key)) {
            array<CMwNod@>@ list;
            m_map.Get(key, @list);

            auto index = list.FindByRef(nod);

            if (index >= 0) {
                list.RemoveAt(index);
            }
        }
    }
}
