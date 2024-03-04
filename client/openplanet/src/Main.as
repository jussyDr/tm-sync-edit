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

    @g_library = Library(library, buttonLabel, buttonPressed, statusText);
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

void OnDestroyed() {
    @g_library = null;
}

class Library {
    private Import::Library@ library;
    private Import::Function@ buttonLabel;
    private Import::Function@ buttonPressed;
    private Import::Function@ statusText;

    Library(
        Import::Library@ library, 
        Import::Function@ buttonLabel, 
        Import::Function@ buttonPressed,
        Import::Function@ statusText
    ) {
        @this.library = library;
        @this.buttonLabel = buttonLabel;
        @this.buttonPressed = buttonPressed;
        @this.statusText = statusText;
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
}
