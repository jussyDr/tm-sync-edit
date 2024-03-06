Library@ g_library = null;

void Main() {
    LoadLibrary();
}

void RenderInterface() {
    if (UI::Begin("Sync Edit")) {
        if (UI::Button("Join")) {
            Join();
        }

        UI::End();
    }
}


void Update(float dt) {
    if (g_library !is null) {
        if (g_library.ShouldJoin()) {
            print("joining");

            g_library.JoinSuccess(true);
        }
    }
}

void OnDestroyed() {
    FreeLibrary();
}

void Join() {
    if (g_library !is null) {
        g_library.Join();
    }
}

void LoadLibrary() {
    auto library = Import::GetLibrary("SyncEdit.dll");

    if (library is null) {
        return;
    }

    auto join = library.GetFunction("Join");

    if (join is null) {
        return;
    }

    auto shouldJoin = library.GetFunction("ShouldJoin");

    if (shouldJoin is null) {
        return;
    }

    auto joinSuccess = library.GetFunction("JoinSuccess");

    if (joinSuccess is null) {
        return;
    }

    @g_library = Library(library, join, shouldJoin, joinSuccess);
}

void FreeLibrary() {
    @g_library = null;
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_join;
    private Import::Function@ m_shouldJoin;
    private Import::Function@ m_joinSuccess;
    
    Library(
        Import::Library@ library,
        Import::Function@ join,
        Import::Function@ shouldJoin,
        Import::Function@ joinSuccess
    ) {
        @m_library = library;
        @m_join = join;
        @m_shouldJoin = shouldJoin;
        @m_joinSuccess = joinSuccess;
    }

    void Join() {
        m_join.Call();
    }

    bool ShouldJoin() {
        return m_shouldJoin.CallBool();
    }

    void JoinSuccess(bool success) {
        m_joinSuccess.Call(success);
    }
}
