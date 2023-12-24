Import::Library@ g_library = null;

void Main() {
    @g_library = Import::GetLibrary("SyncEdit.dll");
}

void OnDestroyed() {
    @g_library = null;
}

void OnEnabled() {
    Main();
}

void OnDisabled() {
    OnDestroyed();
}
