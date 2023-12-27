const string c_title = Icons::Syncthing + " Sync Edit";

[Setting hidden]
bool Setting_WindowEnabled = true;

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
