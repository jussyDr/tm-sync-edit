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

    SerializeBlockInfos(blockInfosValue, "GameCtnBlockInfoClassic");
    SerializeBlockInfos(blockInfosValue, "GameCtnBlockInfoClassic/Deprecated");
    SerializeBlockInfos(blockInfosValue, "GameCtnBlockInfoPillar");

    Json::ToFile(IO::FromDataFolder("BlockInfos.json"), blockInfosValue);
}

void SerializeBlockInfos(Json::Value@ blockInfosValue, const string&in folder) {
    auto fids = Fids::GetGameFolder("GameData/Stadium/GameCtnBlockInfo/" + folder);

    for (uint i = 0; i < fids.Leaves.Length; i++) {
        auto fid = fids.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));

        auto blockInfoValue = Json::Object();
        auto variantsGroundValue = Json::Array();

        if (blockInfo.VariantGround !is null) {
            variantsGroundValue.Add(SerializeBlockInfoVariant(blockInfo.VariantGround));
        }

        for (uint j = 0; j < blockInfo.AdditionalVariantsGround.Length; j++) {
            variantsGroundValue.Add(SerializeBlockInfoVariant(blockInfo.AdditionalVariantsGround[j]));
        }

        auto variantsAirValue = Json::Array();

        if (blockInfo.VariantAir !is null) {
            variantsAirValue.Add(SerializeBlockInfoVariant(blockInfo.VariantAir));
        }

        for (uint j = 0; j < blockInfo.AdditionalVariantsAir.Length; j++) {
            variantsAirValue.Add(SerializeBlockInfoVariant(blockInfo.AdditionalVariantsAir[j]));
        }

        blockInfoValue["variants_ground"] = variantsGroundValue;
        blockInfoValue["variants_air"] = variantsAirValue;

        blockInfosValue[blockInfo.IdName] = blockInfoValue;
    }
}

Json::Value SerializeBlockInfoVariant(CGameCtnBlockInfoVariant@ variant) {
    auto variantValue = Json::Object();

    auto extentValue = Json::Object();
    extentValue["x"] = variant.Size.x - 1;
    extentValue["y"] = variant.Size.y - 1;
    extentValue["z"] = variant.Size.z - 1;

    auto unitsValue = Json::Array();

    for (uint i = 0; i < variant.BlockUnitInfos.Length; i++) {
        auto unit = variant.BlockUnitInfos[i];

        auto unitValue = Json::Object();

        auto offsetValue = Json::Object();
        offsetValue["x"] = unit.Offset.x;
        offsetValue["y"] = unit.Offset.y;
        offsetValue["z"] = unit.Offset.z;

        unitValue["offset"] = offsetValue;

        uint j = 0;

        if (unit.ClipCount_North > 0) {
            unitValue["clip_north"] = SerializeBlockInfoClip(unit.AllClips[j++]);
        }

        if (unit.ClipCount_East > 0) {
            unitValue["clip_east"] = SerializeBlockInfoClip(unit.AllClips[j++]);
        }

        if (unit.ClipCount_South > 0) {
            unitValue["clip_south"] = SerializeBlockInfoClip(unit.AllClips[j++]);
        }

        if (unit.ClipCount_West > 0) {
            unitValue["clip_west"] = SerializeBlockInfoClip(unit.AllClips[j++]);
        }

        unitsValue.Add(unitValue);
    }

    variantValue["extent"] = extentValue;
    variantValue["units"] = unitsValue;

    return variantValue;
}

Json::Value SerializeBlockInfoClip(CGameCtnBlockInfoClip@ clip) {
    if (clip.IsExclusiveFreeClip) {
        auto clipValue = Json::Object();
        auto clipVariantValue = Json::Object();
        clipVariantValue["id"] = clip.IdName;

        if (clip.ASymmetricalClipId.Value != 0xFFFFFFFF) {
            clipVariantValue["asym_clip_id"] = clip.ASymmetricalClipId.GetName();
            clipValue["ExclusiveAsymmetric"] = clipVariantValue;
        } else {
            clipValue["ExclusiveSymmetric"] = clipVariantValue;
        }

        return clipValue;
    }
        
    return Json::Value("NonExclusive");
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
