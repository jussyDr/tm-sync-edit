const string c_libraryName = "SyncEdit.dll";

Library@ LoadLibrary() {
    auto library = Import::GetLibrary(c_libraryName);

    if (library is null) {
        return null;
    }

    auto initFunc = library.GetFunction("Init");

    if (initFunc is null) {
        return null;
    }

    auto destroyFunc = library.GetFunction("Destroy");

    if (destroyFunc is null) {
        return null;
    }

    auto updateFunc = library.GetFunction("Update");

    if (updateFunc is null) {
        return null;
    }

    auto maniaPlanet = cast<CGameManiaPlanet>(GetApp());

    if (maniaPlanet is null) {
        return null;
    }

    auto context = initFunc.CallPointer(maniaPlanet);

    if (context == 0) {
        return null;
    }

    return Library(library, destroyFunc, updateFunc, context);
}

class Library {
    private Import::Library@ m_library;
    private Import::Function@ m_destroyFunc;
    private Import::Function@ m_updateFunc;
    private uint64 m_context;

    Library(Import::Library@ library, Import::Function@ destroyFunc, Import::Function@ updateFunc, uint64 context) {
        @m_library = library;
        @m_destroyFunc = destroyFunc;
        @m_updateFunc = updateFunc;
        m_context = context;
    }

    ~Library() {
        m_destroyFunc.Call(m_context);
    }

    void Update(CGameCtnEditorCommon@ editor) {
        if (editor is null) {
            m_updateFunc.Call(m_context, uint64(0));
        } else {
            m_updateFunc.Call(m_context, editor);
        }
    }
}
