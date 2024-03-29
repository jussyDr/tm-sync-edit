[Setting hidden]
bool Setting_InterfaceVisible = true;

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

const string c_title = "Sync Edit";

Library@ g_library = null;
bool g_connectionOpen = false;

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
        if (g_connectionOpen) {
            UI::LabelText("Host", Setting_Host);

            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                g_library.CloseConnection();
                g_connectionOpen = false;

                g_library.SetStatusText("Canceled");
            }
        } else {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);

            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsNoBlank);

            if (UI::Button("Join")) {
                g_library.OpenConnection(Setting_Host, Setting_Port);
                g_connectionOpen = true;
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
    if (g_library !is null && g_connectionOpen) {
        g_connectionOpen = g_library.UpdateConnection();
    }
}
