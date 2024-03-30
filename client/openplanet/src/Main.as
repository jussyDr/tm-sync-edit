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

    if (g_library is null) {
        return;
    }
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
        } else if (state == State::Connecting || state == State::OpeningMapEditor) {
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
                g_library.OpenConnection(Setting_Host, Setting_Port);
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

    if (state == State::Connected && !IsInMapEditor()) {
        g_library.CloseConnection();
        g_library.SetStatusText("Closed map editor");
        return;
    }

    if (state != State::Disconnected) {
        if (state == State::OpeningMapEditor) {
            OpenMapEditor();
        }

        g_library.UpdateConnection();
    }
}

void OpenMapEditor() {
    auto maniaPlanet = cast<CGameManiaPlanet>(GetApp());
    auto switcher = maniaPlanet.Switcher;

    if (switcher.ModuleStack.Length == 0) {
        return;
    }

    auto currentSwitcherModule = switcher.ModuleStack[switcher.ModuleStack.Length - 1];

    auto mapEditor = cast<CGameCtnEditorFree>(currentSwitcherModule);

    if (mapEditor !is null) {
        g_library.SetMapEditor(mapEditor);
    }   
}

bool IsInMapEditor() {
    auto switcherModules = GetApp().Switcher.ModuleStack;

    for (auto i = 0; i < switcherModules.Length; i++) {
        if (cast<CGameCtnEditorFree>(switcherModules[i]) !is null) {
            return true;
        }
    }

    return false;
}
