[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

Library@ g_library = null;
uint64 g_state = 0;

void Main() {
    @g_library = LoadLibrary();
}

void RenderInterface() {
    if (!UI::Begin("Sync Edit")) {
        return;
    }

    if (g_library !is null) {
        Setting_Host = UI::InputText("Host", Setting_Host);
        Setting_Port = UI::InputText("Port", Setting_Port);

        if (UI::Button("Join")) {
            g_state = g_library.Join();
        }
    }

    UI::End();
}

void Update() {
    if (g_library !is null) {
        if (g_library.Update(g_state)) {
            g_state = 0;
        }
    }
}

void OnDestroyed() {
    @g_library = null;
}

Library@ LoadLibrary() {
    auto inner = Import::GetLibrary("SyncEdit.dll");

    if (inner is null) {
        return null;
    }

    auto join = inner.GetFunction("Join");

    if (join is null) {
        return null;
    }

    auto update = inner.GetFunction("Update");

    if (update is null) {
        return null;
    }

    return Library(inner, join, update);
}

class Library {
    private Import::Library@ m_inner;
    private Import::Function@ m_join;
    private Import::Function@ m_update;

    Library(
        Import::Library@ inner, 
        Import::Function@ join,
        Import::Function@ update,
    ) {
        @m_inner = inner;
        @m_join = join;
        @m_update = update;
    }

    uint64 Join() {
        return m_join.CallPointer();
    }

    bool Update(uint64 state) {
        return m_update.CallBool(state);
    }
}
