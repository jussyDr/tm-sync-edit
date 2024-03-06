[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

Library@ g_library = null;

bool g_joining = false;
string g_joinError = "";

void Main() {
    LoadLibrary();
    LoadAndRegisterObjectModels();
}

void RenderInterface() {
    if (UI::Begin("Sync Edit")) {
        if (g_joining) {
            UI::LabelText("Host", Setting_Host);

            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                CancelJoin();
            }

            UI::Text("Connecting...");
        } else {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);

            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

            if (UI::Button("Join")) {
                Join(Setting_Host, Setting_Port);
            }

            UI::Text(g_joinError);
        }

        UI::End();
    }
}

void Update(float dt) {
    if (g_library !is null) {
        if (g_joining) {
            g_joinError = g_library.JoinError();

            if (g_joinError != "") {
                g_joining = false;
            }
        }

        if (g_joining) {
            if (g_library.OpenMapEditor()) {
                startnew(OpenMapEditor);
            }
        }
    }
}

void OnDestroyed() {
    FreeLibrary();
}

// Registers all in-game block infos and item models, loading them if not yet done.
void LoadAndRegisterObjectModels() {
    if (g_library !is null) {
        auto blockInfoFolder = Fids::GetGameFolder("GameData\\Stadium\\GameCtnBlockInfo");
        LoadAndRegisterBlockInfos(blockInfoFolder);

        auto itemModelFolder = Fids::GetGameFolder("GameData\\Stadium\\Items");
        LoadAndRegisterItemModels(itemModelFolder);
    }
}

void LoadAndRegisterBlockInfos(CSystemFidsFolder@ folder) {
    if (folder is null) {
        return;
    }

    for (uint i = 0; i < folder.Leaves.Length; i++) {
        auto fid = folder.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));

        if (blockInfo !is null) {
            g_library.RegisterBlockInfo(blockInfo.IdName, blockInfo);
        }
    }

    for (uint i = 0; i < folder.Trees.Length; i++) {
        LoadAndRegisterBlockInfos(folder.Trees[i]);
    }
}

void LoadAndRegisterItemModels(CSystemFidsFolder@ folder) {
    if (folder is null) {
        return;
    }

    for (uint i = 0; i < folder.Leaves.Length; i++) {
        auto fid = folder.Leaves[i];
        auto itemModel = cast<CGameItemModel>(Fids::Preload(fid));

        if (itemModel !is null) {
            g_library.RegisterItemModel(itemModel.IdName, itemModel);
        }
    }

    for (uint i = 0; i < folder.Trees.Length; i++) {
        LoadAndRegisterItemModels(folder.Trees[i]);
    }
}

// Join a server with the given host and port.
void Join(const string&in host, const string&in port) {
    if (g_library !is null) {
        g_joinError = "";
        g_library.Join(host, port);
        g_joining = true;
    }
}

// Cancel joining a server.
void CancelJoin() {
    if (g_library !is null) {
        g_joining = false;
        g_library.CancelJoin();
    }
}

// Try to open the map editor (blocking).
void OpenMapEditor() {
    if (g_library !is null) {
        auto maniaPlanet = cast<CGameManiaPlanet>(GetApp());
        auto switcher = maniaPlanet.Switcher;

        if (switcher.ModuleStack.Length == 1 && cast<CTrackManiaMenus>(switcher.ModuleStack[0]) !is null) {
            maniaPlanet.ManiaTitleControlScriptAPI.EditNewMap2("Stadium", "48x48Screen155Day", "", "CarSport", "", false, "", "");
        }

        while (switcher.ModuleStack.Length != 1 || cast<CGameCtnEditorFree>(switcher.ModuleStack[0]) is null) {
            yield();
        }

        g_library.OpenMapEditorResult(true);
    }
}
