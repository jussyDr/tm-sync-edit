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

    auto openEditor = library.GetFunction("OpenEditor");

    if (openEditor is null) {
        return;
    }

    auto openEditorResult = library.GetFunction("OpenEditorResult");

    if (openEditorResult is null) {
        return;
    }

    @g_library = Library(library, join, cancelJoin, joinError, openEditor, openEditorResult);
}

void FreeLibrary() {
    @g_library = null;
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_join;
    private Import::Function@ m_cancelJoin;
    private Import::Function@ m_joinError;
    private Import::Function@ m_openEditor;
    private Import::Function@ m_openEditorResult;
    
    Library(
        Import::Library@ library,
        Import::Function@ join,
        Import::Function@ cancelJoin,
        Import::Function@ joinError,
        Import::Function@ openEditor,
        Import::Function@ openEditorResult
    ) {
        @m_library = library;
        @m_join = join;
        @m_cancelJoin = cancelJoin;
        @m_joinError = joinError;
        @m_openEditor = openEditor;
        @m_openEditorResult = openEditorResult;
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

    bool OpenEditor() {
        return m_openEditor.CallBool();
    }

    void OpenEditorResult(bool success) {
        m_openEditorResult.Call(success);
    }
}
