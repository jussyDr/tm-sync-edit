const string c_title = Icons::Syncthing + " Sync Edit";

[Setting hidden]
bool Setting_WindowEnabled = true;

CGameCtnApp@ g_app = null;
Import::Library@ g_library = null;

void Main() {
    @g_app = GetApp();

    @g_library = Import::GetLibrary("SyncEdit.dll");
    auto libraryInit = g_library.GetFunction("Init");
    libraryInit.Call();
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

    UI::End();
}

void Update(float dt) {
    
}

void OnDestroyed() {
    auto libraryDestroy = g_library.GetFunction("Destroy");
    libraryDestroy.Call();
    @g_library = null;
}
