// Interface for calling functions in the SyncEdit.dll library.

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

    auto openMapEditor = library.GetFunction("OpenMapEditor");

    if (openMapEditor is null) {
        return;
    }

    auto openMapEditorResult = library.GetFunction("OpenMapEditorResult");

    if (openMapEditorResult is null) {
        return;
    }

    @g_library = Library(library, join, cancelJoin, joinError, openMapEditor, openMapEditorResult);
}

void FreeLibrary() {
    @g_library = null;
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_join;
    private Import::Function@ m_cancelJoin;
    private Import::Function@ m_joinError;
    private Import::Function@ m_openMapEditor;
    private Import::Function@ m_openMapEditorResult;
    
    Library(
        Import::Library@ library,
        Import::Function@ join,
        Import::Function@ cancelJoin,
        Import::Function@ joinError,
        Import::Function@ openMapEditor,
        Import::Function@ openMapEditorResult
    ) {
        @m_library = library;
        @m_join = join;
        @m_cancelJoin = cancelJoin;
        @m_joinError = joinError;
        @m_openMapEditor = openMapEditor;
        @m_openMapEditorResult = openMapEditorResult;
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

    bool OpenMapEditor() {
        return m_openMapEditor.CallBool();
    }

    void OpenMapEditorResult(bool success) {
        m_openMapEditorResult.Call(success);
    }
}
