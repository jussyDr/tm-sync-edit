const string c_pluginTitle = "Sync Edit";

[Setting hidden]
bool Setting_InterfaceVisible = true;

Library@ g_library = null;

void Main() {
    @g_library = LoadLibrary();

    // cast<CGameManiaPlanet>(GetApp()).ManiaTitleControlScriptAPI.EditNewMap2("Stadium", "48x48Screen155Day", "", "CarSport", "", false, "", "");
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
        if (UI::Button("Join")) {
           
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
        auto editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

        g_library.Update(editor);
    }
}

void OnDestroyed() {
    @g_library = null;
}
