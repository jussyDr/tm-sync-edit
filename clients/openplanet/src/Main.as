const string c_title = Icons::Syncthing + " Sync Edit";

bool g_interfaceVisible = true;
string g_host = "127.0.0.1";
string g_port = "8369";
bool g_disconnected = true;

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

    auto placeBlockHook = Dev::Hook(pfPlaceBlock, 0, "OnPlaceBlock");
    auto removeBlockHook = Dev::Hook(pfRemoveBlock, 0, "OnRemoveBlock");
    auto placeItemHook = Dev::Hook(pfPlaceItem, 0, "OnPlaceItem");
    auto removeItemHook = Dev::Hook(pfRemoveItem, 0, "OnRemoveItem");

    Dev::Unhook(placeBlockHook);
    Dev::Unhook(removeBlockHook);
    Dev::Unhook(placeItemHook);
    Dev::Unhook(removeItemHook);

    auto editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

    CGameCtnBlockInfo@ blockInfo;
    blockInfos.Get("RoadTechStraight", @blockInfo);

    PlaceBlock(fnPlaceBlock, pfPlaceBlock, editor, blockInfo, 20, 20, 20, CGameEditorPluginMap::ECardinalDirections::North, false, false, CGameEditorPluginMap::EMapElemColor::Default);
}

void RenderInterface() {
    if (Setting_InterfaceVisible) {
        if (UI::Begin(c_title)) {
            if (g_disconnected) {
                g_host = UI::InputText("Host", g_host, UI::InputTextFlags::CharsNoBlank);
                g_port = UI::InputText("Port", g_port, UI::InputTextFlags::CharsDecimal);

                if (g_disconnected) {
                    if (UI::Button("Join")) {
                        startnew(Join);
                    }
                }

                UI::Text("Disconnected");
            }
            
            UI::End();
        }
    }
}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", Setting_InterfaceVisible)) {
        Setting_InterfaceVisible = !Setting_InterfaceVisible;
    }
}

void OnDestroyed() {

}

void OnPlaceBlock() {
    print("placed block");
}

void OnRemoveBlock() {
    print("removed block");
}

void OnPlaceItem() {
    print("placed item");
}

void OnRemoveItem() {
    print("removed item");
}

void Join() {

}

void LoadBlockInfos(dictionary@ blockInfos, const string&in folder) {
    auto fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/" + folder);

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfos[blockInfo.IdName] = @blockInfo;
    }
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
