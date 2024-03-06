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

    auto joinError = library.GetFunction("JoinError");

    if (joinError is null) {
        return;
    }

    @g_library = Library(library, join, cancelJoin, joinError);
}

void FreeLibrary() {
    @g_library = null;
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_join;
    private Import::Function@ m_cancelJoin;
    private Import::Function@ m_joinError;
    
    Library(
        Import::Library@ library,
        Import::Function@ join,
        Import::Function@ cancelJoin,
        Import::Function@ joinError
    ) {
        @m_library = library;
        @m_join = join;
        @m_cancelJoin = cancelJoin;
        @m_joinError = joinError;
    }

    void Join(const string&in host, const string&in port) {
        m_join.Call(host, port);
    }

    void CancelJoin() {
        m_cancelJoin.Call();
    }

    string JoinError() {
        return m_joinError.CallString();
    }
}
