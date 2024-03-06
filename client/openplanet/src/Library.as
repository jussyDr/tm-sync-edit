void LoadLibrary() {
    auto library = Import::GetLibrary("SyncEdit.dll");

    if (library is null) {
        return;
    }

    auto join = library.GetFunction("Join");

    if (join is null) {
        return;
    }

    auto cancelJoin = library.GetFunction("CancelJoin");

    if (cancelJoin is null) {
        return;
    }

    @g_library = Library(library, join, cancelJoin);
}

void FreeLibrary() {
    @g_library = null;
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_join;
    private Import::Function@ m_cancelJoin;
    
    Library(
        Import::Library@ library,
        Import::Function@ join,
        Import::Function@ cancelJoin
    ) {
        @m_library = library;
        @m_join = join;
        @m_cancelJoin = cancelJoin;
    }

    void Join(const string&in host, const string&in port) {
        m_join.Call(host, port);
    }

    void CancelJoin() {
        m_cancelJoin.Call();
    }
}
