[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

Library@ g_library = null;

bool g_joining = false;

void Main() {
    LoadLibrary();
}

void RenderInterface() {
    if (UI::Begin("Sync Edit")) {
        if (g_joining) {
            UI::LabelText("Host", Setting_Host);

            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                CancelJoin();
            }
        } else {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);

            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

            if (UI::Button("Join")) {
                Join(Setting_Host, Setting_Port);
            }
        }

        UI::End();
    }
}

void Update(float dt) {

}

void OnDestroyed() {
    FreeLibrary();
}

void Join(const string&in host, const string&in port) {
    if (g_library !is null) {
        g_library.Join(host, port);
        g_joining = true;
    }
}

void CancelJoin() {
    if (g_library !is null) {
        g_joining = false;
        g_library.CancelJoin();
    }
}
