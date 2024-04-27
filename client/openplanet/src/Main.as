[Setting hidden]
bool Setting_InterfaceVisible = true;

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

const string c_title = "Sync Edit";

Library@ g_library = null;

void Main() {
    @g_library = LoadLibrary();
}

void RenderInterface() {
    if (!Setting_InterfaceVisible) {
        return;
    }

    if (!UI::Begin(c_title)) {
        return;
    }

    if (g_library !is null) {
        auto state = g_library.GetState();

        if (state == State::Connected) {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Exit")) {
                g_library.CloseConnection();
                g_library.SetStatusText("Disconnected");
            }
        } else if (state == State::Connecting) {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                g_library.CloseConnection();
                g_library.SetStatusText("Canceled");
            }
        } else if (state == State::Disconnected) {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);
            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsNoBlank);

            if (UI::Button("Join")) {
                g_library.OpenConnection(Setting_Host, Setting_Port, Fids::GetGameFolder(""));
            }
        }

        UI::Text(g_library.GetStatusText());
    }

    UI::End();
}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", Setting_InterfaceVisible)) {
        Setting_InterfaceVisible = !Setting_InterfaceVisible;
    }
}

void Update(float dt) {
    if (g_library is null) {
        return;
    }

    auto state = g_library.GetState();

    if (state == State::Connected && !IsMapEditorOpen()) {
        g_library.CloseConnection();
        g_library.SetStatusText("Closed map editor");
        return;
    }

    if (g_library.ShouldOpenEditor()) {
        OpenMapEditor();
    }

    if (state != State::Disconnected) {
        g_library.UpdateConnection();
    }
}

void OpenMapEditor() {
    auto maniaPlanet = cast<CGameManiaPlanet>(GetApp());
    auto switcherModules = maniaPlanet.Switcher.ModuleStack;

    if (switcherModules.Length == 0) {
        return;
    }

    auto currentSwitcherModule = switcherModules[switcherModules.Length - 1];

    if (cast<CGameCtnEditorFree>(currentSwitcherModule) !is null) {
        g_library.SetMapEditor(cast<CGameCtnEditorFree>(currentSwitcherModule));
        return;
    }   

    auto isMapEditorOpen = false;

    for (uint i = 0; i < switcherModules.Length - 1; i++) {
        if (cast<CGameCtnEditorFree>(switcherModules[i]) !is null) {
            isMapEditorOpen = true;
            break;
        }
    }

    if (isMapEditorOpen) {
        if (cast<CGameEditorMediaTracker>(currentSwitcherModule) !is null) {
            cast<CGameEditorMediaTrackerPluginAPI>(cast<CGameEditorMediaTracker>(currentSwitcherModule).PluginAPI).Quit();
            return;
        } else if (cast<CGameEditorItem>(currentSwitcherModule) !is null) {
            cast<CGameEditorItem>(currentSwitcherModule).Exit();
            return;
        }
    } else {
        if (cast<CTrackManiaMenus>(currentSwitcherModule) !is null) {
            maniaPlanet.ManiaTitleControlScriptAPI.EditNewMap2("Stadium", "48x48Screen155Day", "", "CarSport", "", false, "", "");
            return;
        }
    }

    maniaPlanet.BackToMainMenu();
}

bool IsMapEditorOpen() {
    auto switcherModules = GetApp().Switcher.ModuleStack;

    for (uint i = 0; i < switcherModules.Length; i++) {
        if (cast<CGameCtnEditorFree>(switcherModules[i]) !is null) {
            return true;
        }
    }

    return false;
}
