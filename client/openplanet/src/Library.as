// Interface for calling functions in the SyncEdit.dll library.

void LoadLibrary() {
    auto library = Import::GetLibrary("SyncEdit.dll");

    if (library is null) {
        return;
    }

    auto registerBlockInfo = library.GetFunction("RegisterBlockInfo");

    if (registerBlockInfo is null) {
        return;
    }

    auto registerItemModel = library.GetFunction("RegisterItemModel");

    if (registerItemModel is null) {
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

    auto joinStatus = library.GetFunction("JoinStatus");

    if (joinStatus is null) {
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

    @g_library = Library(library, registerBlockInfo, registerItemModel, join, cancelJoin, joinError, joinStatus, openMapEditor, openMapEditorResult);
}

void FreeLibrary() {
    @g_library = null;
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_registerBlockInfo;
    private Import::Function@ m_registerItemModel;
    private Import::Function@ m_join;
    private Import::Function@ m_cancelJoin;
    private Import::Function@ m_joinError;
    private Import::Function@ m_joinStatus;
    private Import::Function@ m_openMapEditor;
    private Import::Function@ m_openMapEditorResult;
    
    Library(
        Import::Library@ library,
        Import::Function@ registerBlockInfo,
        Import::Function@ registerItemModel,
        Import::Function@ join,
        Import::Function@ cancelJoin,
        Import::Function@ joinError,
        Import::Function@ joinStatus,
        Import::Function@ openMapEditor,
        Import::Function@ openMapEditorResult
    ) {
        @m_library = library;
        @m_registerBlockInfo = registerBlockInfo;
        @m_registerItemModel = registerItemModel;
        @m_join = join;
        @m_cancelJoin = cancelJoin;
        @m_joinError = joinError;
        @m_joinStatus = joinStatus;
        @m_openMapEditor = openMapEditor;
        @m_openMapEditorResult = openMapEditorResult;
    }

    void RegisterBlockInfo(const string&in id, const CGameCtnBlockInfo@ blockInfo) {
        m_registerBlockInfo.Call(id, blockInfo);
    }

    void RegisterItemModel(const string&in id, const CGameItemModel@ itemModel) {
        m_registerItemModel.Call(id, itemModel);
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

    string JoinStatus() {
        return m_joinStatus.CallString();
    }

    bool OpenMapEditor() {
        return m_openMapEditor.CallBool();
    }

    void OpenMapEditorResult(bool success) {
        m_openMapEditorResult.Call(success);
    }
}
