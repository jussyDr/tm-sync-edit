const string c_libraryFileName = "SyncEdit.dll";

Library@ LoadLibrary() {
    auto importLibrary = Import::GetLibrary(c_libraryFileName);

    if (importLibrary is null) {
        return null;
    }

    auto createContext = importLibrary.GetFunction("CreateContext");

    if (createContext is null) {
        return null;
    }

    auto destroyContext = importLibrary.GetFunction("DestroyContext");

    if (destroyContext is null) {
        return null;
    }

    auto openConnection = importLibrary.GetFunction("OpenConnection");

    if (openConnection is null) {
        return null;
    }

    auto updateConnection = importLibrary.GetFunction("UpdateConnection");

    if (updateConnection is null) {
        return null;
    }

    auto closeConnection = importLibrary.GetFunction("CloseConnection");

    if (closeConnection is null) {
        return null;
    }

    auto gameFolder = Fids::GetGameFolder("");
    auto gameDataFolder = FindSubfolder(gameFolder, "GameData");
    auto stadiumFolder = FindSubfolder(gameDataFolder, "Stadium");
    auto blockInfoFolder = FindSubfolder(stadiumFolder, "GameCtnBlockInfo");
    auto itemsFolder = FindSubfolder(stadiumFolder, "Items");
    PreloadAllFidsInFolder(blockInfoFolder);
    PreloadAllFidsInFolder(itemsFolder);

    auto context = createContext.CallPointer(gameFolder);

    if (context == 0) {
        return null;
    }

    return Library(importLibrary, context, destroyContext, openConnection, updateConnection, closeConnection);
}

CSystemFidsFolder@ FindSubfolder(CSystemFidsFolder@ folder, const string&in name) {
    for (uint i = 0; i < folder.Trees.Length; i++) {
        if (folder.Trees[i].DirName == name) {
            return folder.Trees[i];
        }
    }

    return null;
}

void PreloadAllFidsInFolder(CSystemFidsFolder@ folder) {
    for (uint i = 0; i < folder.Leaves.Length; i++) {
        Fids::Preload(folder.Leaves[i]);
    }

    yield();

    for (uint i = 0; i < folder.Trees.Length; i++) {
        PreloadAllFidsInFolder(folder.Trees[i]);
    }
}

class Library {
    private Import::Library@ m_importLibrary;
    private uint64 m_context; 
    private Import::Function@ m_destroyContext;
    private Import::Function@ m_openConnection;
    private Import::Function@ m_updateConnection;
    private Import::Function@ m_closeConnection;

    Library(
        Import::Library@ importLibrary, 
        uint64 context,
        Import::Function@ destroyContext,
        Import::Function@ openConnection,
        Import::Function@ updateConnection,
        Import::Function@ closeConnection
    ) {
        @m_importLibrary = importLibrary;
        m_context = context;
        @m_destroyContext = destroyContext;
        @m_openConnection = openConnection;
        @m_updateConnection = updateConnection;
        @m_updateConnection = updateConnection;
        @m_closeConnection = closeConnection;
    }

    ~Library() {
        m_destroyContext.Call(m_context);
    }

    State GetState() {
        return State(Dev::ReadUInt8(m_context));
    }

    string GetStatusText() {
        return Dev::ReadCString(Dev::ReadUInt64(m_context + 8));
    }

    void SetStatusText(const string&in statusText) {
        if (statusText.Length >= 256) {
            return;
        }

        Dev::WriteCString(Dev::ReadUInt64(m_context + 8), statusText);
        Dev::Write(Dev::ReadUInt64(m_context + 8) + statusText.Length, uint8(0));
    }

    void SetMapEditor(CGameCtnEditorFree@ mapEditor) {
        Dev::Write(m_context + 16, Dev::ForceCast<uint64>(mapEditor).Get());
    }

    void OpenConnection(const string&in host, const string&in port) {
        m_openConnection.Call(m_context, host, port);
    }

    void UpdateConnection() {
        m_updateConnection.Call(m_context);
    }

    void CloseConnection() {
        m_closeConnection.Call(m_context);
    }
}

enum State {
    Disconnected,
    Connecting,
    OpeningMapEditor,
    Connected,
}
