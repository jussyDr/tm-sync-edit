const string c_pluginTitle = "Sync Edit";

[Setting hidden]
bool Setting_InterfaceVisible = true;

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

Library@ g_library = null;

void Main() {
    @g_library = LoadLibrary();
}

void RenderInterface() {
    if (!Setting_InterfaceVisible) {
        return;
    }

    if (!UI::Begin(c_pluginTitle)) {
        return;
    }

    if (g_library is null) {
        UI::PushStyleColor(UI::Col::Text, vec4(1, 0, 0, 1));
        UI::Text("Error: Failed to load library '" + c_libraryName + "'");
        UI::PopStyleColor();
    } else {
        Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);

        Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

        if (UI::Button("Join")) {
           g_library.Join(Setting_Host, Setting_Port);
        }
    }

    UI::End();
}

void RenderMenu() {
    if (UI::MenuItem(c_pluginTitle, "", Setting_InterfaceVisible)) {
        Setting_InterfaceVisible = !Setting_InterfaceVisible;
    }
}

void Update(float dt) {
    if (g_library !is null) {
        g_library.Update();
    }
}

void OnDestroyed() {
    @g_library = null;
}
