[Setting hidden]
bool Setting_WindowEnabled = true;

Import::Library@ g_library = null;

void Main() {
    @g_library = Import::GetLibrary("Test.dll");
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

    if (!UI::Begin("Sync Edit", open)) {
        return;
    }

    Setting_WindowEnabled = open;

    UI::Text("Hello!");

    UI::End();
}

void RenderMenu() {
    if (UI::MenuItem("Sync edit", "", Setting_WindowEnabled)) {
        Setting_WindowEnabled = !Setting_WindowEnabled;
    }
}
