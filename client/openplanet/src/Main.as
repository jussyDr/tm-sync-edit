const string c_title = Icons::Syncthing + " Sync Edit";

[Setting hidden]
bool Setting_WindowEnabled = true;

[Setting hidden]
string Setting_Host = "0.0.0.0";

[Setting hidden]
string Setting_Port = "8369";

Import::Library@ g_library = null;
Import::Function@ g_libraryJoin = null;

void Main() {}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", Setting_WindowEnabled)) {
        Setting_WindowEnabled = !Setting_WindowEnabled;
    }
}

void RenderInterface() {
    if (!Setting_WindowEnabled) {
        return;
    }

    if (!UI::Begin(c_title, Setting_WindowEnabled)) {
        return;
    }

    Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);
    Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

    if (UI::Button("Join")) {
        @g_library = Import::GetLibrary("SyncEdit.dll");

        if (g_library != null) {
            @g_libraryJoin = g_library.GetFunction("Join");
        }

        if (g_libraryJoin != null) {
            uint16 port = Text::ParseUInt(Setting_Port);
            g_libraryJoin.Call(Setting_Host, port);
        }
    }

    UI::End();
}

void Update(float dt) {}

void OnDestroyed() {}
