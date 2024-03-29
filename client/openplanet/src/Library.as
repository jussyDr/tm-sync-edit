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

    auto context = createContext.CallPointer();

    if (context == 0) {
        return null;
    }

    return Library(importLibrary, context, destroyContext);
}

class Library {
    private Import::Library@ m_importLibrary;
    private uint64 m_context; 
    private Import::Function@ m_destroyContext;

    Library(
        Import::Library@ importLibrary, 
        uint64 context,
        Import::Function@ destroyContext
    ) {
        @m_importLibrary = importLibrary;
        m_context = context;
        @m_destroyContext = destroyContext;
    }

    ~Library() {
        m_destroyContext.Call(m_context);
    }
}
