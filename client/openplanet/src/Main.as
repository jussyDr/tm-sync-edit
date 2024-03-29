[Setting hidden]
bool Setting_InterfaceVisible = true;

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

const string c_title = "Sync Edit";

Library@ g_library = null;
uint64 g_context = 0;

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
    
    UI::End();
}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", Setting_InterfaceVisible)) {
        Setting_InterfaceVisible = !Setting_InterfaceVisible;
    }
}

void Update() {

}

void OnDestroyed() {
 
}
