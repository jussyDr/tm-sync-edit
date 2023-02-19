const string c_title = Icons::Syncthing + " Sync Edit";

enum State {
    Disconnected,
    Connecting,
    Connected,
}

bool g_interfaceVisible;
string g_status;

State g_state;
bool g_stopped;
Net::Socket@ g_socket;

Dev::HookInfo@ g_hookPlaceBlock;
Dev::HookInfo@ g_hookRemoveBlock;
Dev::HookInfo@ g_hookPlaceItem;
Dev::HookInfo@ g_hookRemoveItem;

uint g_numBlocksPlaced;
uint g_numItemsPlaced;

void Main() {
    g_status = "Disconnected";
    g_state = State::Disconnected;
    g_stopped = false;
    g_numBlocksPlaced = 0;
    g_numItemsPlaced = 0;
}

void RenderInterface() {
    if (UI::Begin(c_title)) {
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
    if (UI::MenuItem(c_title, "", g_interfaceVisible)) {
        g_interfaceVisible = !g_interfaceVisible;
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
    g_numBlocksPlaced++;
}

void OnRemoveBlock(CGameCtnBlock@ rdx) {
    if (g_numBlocksPlaced > 0) {
        g_numBlocksPlaced--;
    } else {
        if (Block::IsFree(rdx)) {
            auto freeBlockValue = SerializeFreeBlock(rdx);
            SendCommand("RemoveFreeBlock", freeBlockValue);
        } else {
            auto blockValue = SerializeBlock(rdx);
            SendCommand("RemoveBlock", blockValue);
        }
    }
}

void OnPlaceItem() {
    g_numItemsPlaced++;
}

void OnRemoveItem(CGameCtnAnchoredObject@ rdx) {
    auto itemValue = SerializeItem(rdx);
    SendCommand("RemoveItem", itemValue);
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

    auto library = Import::GetLibrary("SyncEdit.dll");

    if (library is null) {
        Error("Failed to find library 'SyncEdit.dll'");
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

            while (g_numBlocksPlaced > 0) {
                auto block = blocks[blocks.Length - g_numBlocksPlaced];

                if (Block::IsFree(block)) {
                    auto freeBlockValue = SerializeFreeBlock(block);
                    SendCommand("PlaceFreeBlock", freeBlockValue);
                } else {
                    auto blockValue = SerializeBlock(block);
                    SendCommand("PlaceBlock", blockValue);
                }

                g_numBlocksPlaced--;
            }

            auto items = editor.Challenge.AnchoredObjects;

            while (g_numItemsPlaced > 0) {
                auto item = items[items.Length - g_numItemsPlaced];
                auto itemValue = SerializeItem(item);
                SendCommand("PlaceItem", itemValue);

                g_numItemsPlaced--;
            }
        }

        uint available = g_socket.Available();

        if (available == 0) {
            Error("Server disconnected");
            return;
        }

        while (available > 0) {
            if (available < 4) {
                Error("Disconnected");
                return;
            }

            uint frameLength = g_socket.ReadUint32();
            available -= 4;

            if (available < frameLength) {
                Error("Disconnected");
                return;
            }

            auto json = g_socket.ReadRaw(frameLength);
            available -= frameLength;

            auto commandValue = Json::Parse(json);

            if (commandValue.HasKey("PlaceBlock")) {
                print("received: PlaceBlock");
            } else if (commandValue.HasKey("RemoveBlock")) {
                print("received: RemoveBlock");
            } else if (commandValue.HasKey("PlaceFreeBlock")) {
                print("received: PlaceFreeBlock");
            } else if (commandValue.HasKey("RemoveFreeBlock")) {
                print("received: RemoveFreeBlock");
            } else if (commandValue.HasKey("PlaceItem")) {
                print("received: PlaceItem");
            } else if (commandValue.HasKey("RemoveItem")) {
                print("received: RemoveItem");
            }
        }
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
    blockValue["variant_index"] = 0;
    blockValue["is_ghost"] = Block::Flags(block) & 0x10 != 0;
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

namespace Block {
    uint8 Flags(CGameCtnBlock@ block) {
        auto pBlock = Dev::ForceCast<uint64>(block).Get();

        return Dev::ReadUInt8(pBlock + 135);
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
        bool isGhost,
        CGameEditorPluginMap::EMapElemColor color
    ) {
        auto nod = placeBlockFunc.CallNod(
            pfPlaceBlock,
            editor,
            blockInfo,
            int3(x, y, z),
            dir,
            isGround, 
            isGhost,
            color
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
            color
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
            color,
            animOffset
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
