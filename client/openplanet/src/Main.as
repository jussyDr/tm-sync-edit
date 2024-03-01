const string c_title = Icons::Syncthing + " Sync Edit";

[Setting hidden]
bool Setting_WindowEnabled = true;

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

Import::Library@ g_library = null;
Import::Function@ g_libraryUpdate = null;
Import::Function@ g_libraryDestroy = null;

void Main() {
 
}

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
        Join();
    }

    UI::End();
}

void Update(float dt) {
    if (g_libraryUpdate != null) {
        g_libraryUpdate.Call();
    }
}

void OnDestroyed() {
    g_libraryDestroy.Call();
    @g_library = null;
}

void Join() {
    @g_library = Import::GetLibrary("SyncEdit.dll");

    if (g_library == null) {
        return;
    }

    auto libraryJoin = g_library.GetFunction("Join");
    @g_libraryUpdate = g_library.GetFunction("Update");
    @g_libraryDestroy = g_library.GetFunction("Destroy");


    uint16 port = Text::ParseUInt(Setting_Port);

    libraryJoin.Call(Setting_Host, port);
}
