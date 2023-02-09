void RenderMenu() {
    if (UI::Button("Extract Block Infos")) {
        ExtractBlockInfos();
    }

    if (UI::Button("Extract Item Model Ids")) {
        ExtractItemModelIds();
    }
}

void ExtractBlockInfos() {
    auto blockInfosValue = Json::Object();

    auto fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/GameCtnBlockInfoClassic");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfosValue[blockInfo.IdName] = ExtractBlockInfo(blockInfo);
    }

    @fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/GameCtnBlockInfoClassic/Deprecated");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfosValue[blockInfo.IdName] = ExtractBlockInfo(blockInfo);
    }

    @fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/GameCtnBlockInfoPillar");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        blockInfosValue[blockInfo.IdName] = ExtractBlockInfo(blockInfo);
    }

    Json::ToFile(IO::FromDataFolder("BlockInfos.json"), blockInfosValue);
}

Json::Value ExtractBlockInfo(CGameCtnBlockInfo@ blockInfo) {
    return Json::Value();
}

void ExtractItemModelIds() {
    auto itemModelIdsValue = Json::Array();

    auto fids = Fids::GetGameFolder("GameData/Stadium/Items");

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto itemModel = cast<CGameItemModel>(Fids::Preload(fid));
        itemModelIdsValue.Add(itemModel.IdName);
    }

    Json::ToFile(IO::FromDataFolder("ItemModelIds.json"), itemModelIdsValue);
}
