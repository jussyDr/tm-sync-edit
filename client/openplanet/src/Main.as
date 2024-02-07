const string c_title = Icons::Syncthing + " Sync Edit";

Import::Library@ g_library;
Import::Function@ g_libraryUpdate;

[Setting hidden]
bool Setting_WindowEnabled = true;

void Main() {
    @g_library = Import::GetLibrary("SyncEdit.dll");

    if (g_library == null) {
        return;
    }

    @g_libraryUpdate = g_library.GetFunction("update");
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

    bool open;

    if (!UI::Begin(c_title, open)) {
        return;
    }

    Setting_WindowEnabled = open;

    UI::End();
}

void Update(float dt) {
    if (@g_libraryUpdate != null) {
        g_libraryUpdate.Call();
    }
}

void OnDestroyed() {
    @g_libraryUpdate = null;
    @g_library = null;
}
