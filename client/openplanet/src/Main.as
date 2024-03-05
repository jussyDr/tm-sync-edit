const string c_title = "Sync Edit";
const string c_libraryPath = "SyncEdit.dll";

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

Library@ g_library = null;

void Main() {
    auto library = Import::GetLibrary(c_libraryPath);

    if (library is null) {
        return;
    }

    auto buttonLabel = library.GetFunction("ButtonLabel");

    if (buttonLabel is null) {
        return;
    }

    auto buttonPressed = library.GetFunction("ButtonPressed");

    if (buttonPressed is null) {
        return;
    }

    auto statusText = library.GetFunction("StatusText");

    if (statusText is null) {
        return;
    }

    auto setEditor = library.GetFunction("SetEditor");

    if (setEditor is null) {
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

    @g_library = Library(library, buttonLabel, buttonPressed, statusText, setEditor, registerBlockInfo, registerItemModel);

    LoadBlockInfos();
    LoadItemModels();
}

void RenderInterface() {
    if (UI::Begin(c_title)) {
        if (g_library is null) {
            UI::Text("failed to load library: " + c_libraryPath);
        } else {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);

            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsNoBlank);

            if (UI::Button(g_library.ButtonLabel())) {
                g_library.ButtonPressed(Setting_Host, Setting_Port);
            }

            UI::Text(g_library.StatusText());
        }

        UI::End();
    }
}

void Update(float dt) {
    if (g_library !is null) {
        auto editor = GetApp().Editor;
        g_library.SetEditor(editor);
    }
}

void OnDestroyed() {
    @g_library = null;
}

void LoadBlockInfos() {
    if (g_library !is null) {
        auto folder = Fids::GetGameFolder("GameData\\Stadium\\GameCtnBlockInfo");
        LoadBlockInfosInner(folder);
    }
}

void LoadBlockInfosInner(CSystemFidsFolder@ folder) {
    if (folder is null) {
        return;
    }

    for (uint32 i = 0; i < folder.Leaves.Length; i++) {
        auto fid = folder.Leaves[i];
        auto blockInfo = cast<CGameCtnBlockInfo>(Fids::Preload(fid));
        g_library.RegisterBlockInfo(blockInfo.IdName, blockInfo);

        if (i % 500 == 0) {
            yield();
        }
    }

    for (uint32 i = 0; i < folder.Trees.Length; i++) {
        LoadBlockInfosInner(folder.Trees[i]);
    }
}

void LoadItemModels() {
    if (g_library !is null) {
        auto folder = Fids::GetGameFolder("GameData\\Stadium\\Items");
        LoadItemModelsInner(folder);
    }
}

void LoadItemModelsInner(CSystemFidsFolder@ folder) {
    if (folder is null) {
        return;
    }

    for (uint32 i = 0; i < folder.Leaves.Length; i++) {
        auto fid = folder.Leaves[i];
        auto itemModel = cast<CGameItemModel>(Fids::Preload(fid));
        g_library.RegisterItemModel(itemModel.IdName, itemModel);

        if (i % 500 == 0) {
            yield();
        }
    }

    for (uint32 i = 0; i < folder.Trees.Length; i++) {
        LoadItemModelsInner(folder.Trees[i]);
    }
}

class Library {
    private Import::Library@ library;
    private Import::Function@ buttonLabel;
    private Import::Function@ buttonPressed;
    private Import::Function@ statusText;
    private Import::Function@ setEditor;
    private Import::Function@ registerBlockInfo;
    private Import::Function@ registerItemModel;

    Library(
        Import::Library@ library, 
        Import::Function@ buttonLabel, 
        Import::Function@ buttonPressed,
        Import::Function@ statusText,
        Import::Function@ setEditor,
        Import::Function@ registerBlockInfo,
        Import::Function@ registerItemModel
    ) {
        @this.library = library;
        @this.buttonLabel = buttonLabel;
        @this.buttonPressed = buttonPressed;
        @this.statusText = statusText;
        @this.setEditor = setEditor;
        @this.registerBlockInfo = registerBlockInfo;
        @this.registerItemModel = registerItemModel;
    }

    string ButtonLabel() {
        return buttonLabel.CallString();
    }

    void ButtonPressed(const string&in host, const string&in port) {
        buttonPressed.Call(host, port);
    }

    string StatusText() {
        return statusText.CallString();
    }

    void SetEditor(const CGameCtnEditor@ editor) {
        if (editor is null) {
            setEditor.Call(uint64(0));
        } else {
            setEditor.Call(editor);
        }
    }

    void RegisterBlockInfo(const string&in id, const CGameCtnBlockInfo@ blockInfo) {
        registerBlockInfo.Call(id, blockInfo);
    }

    void RegisterItemModel(const string&in id, const CGameItemModel@ itemModel) {
        registerItemModel.Call(id, itemModel);
    }
}
