const string c_title = Icons::Syncthing + " Sync Edit";

[Setting hidden]
bool Setting_WindowEnabled = true;

Import::Library@ g_library = null;
Import::Function@ g_function = null;

void Main() {
    @g_library = Import::GetLibrary("SyncEdit.dll");

    if (g_library == null) {
        return;
    }

    @g_function = g_library.GetFunction("Run");
}

void OnDestroyed() {
    @g_library = null;
}

void OnEnabled() {
    
}

void OnDisabled() {
    
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

    if (g_library == null) {
        UI::Text("Failed to open SyncEdit.dll");
    } else {
        if (g_function == null) {
            UI::Text("Failed to load function");
        } else {
            if (UI::Button("Test")) {
                g_function.Call();
            }
        }
    }

    UI::End();
}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", Setting_WindowEnabled)) {
        Setting_WindowEnabled = !Setting_WindowEnabled;
    }
}
