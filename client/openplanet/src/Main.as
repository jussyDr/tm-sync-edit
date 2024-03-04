const string c_title = "Sync Edit";

[Setting hidden]
string Setting_Host = "127.0.0.1";

[Setting hidden]
string Setting_Port = "8369";

string g_errorString = "";

Library@ g_library = null;

void RenderInterface() {
    if (!UI::Begin(c_title)) {
        return;
    }

    Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);

    Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

    if (UI::Button("Join")) {
        LoadLibrary();

        if (g_library !is null) {
            if (Setting_Port == "") {
                g_errorString = "no port specified";
            } else {
                auto port = Text::ParseUInt(Setting_Port);

                if (port == 0 || port > 65535) {
                    g_errorString = "invalid port";
                } else {
                    if (!g_library.Join(Setting_Host, port)) {
                        g_errorString = g_library.LastErrorString();
                    }
                }
            }
        }
    }

    UI::Text(g_errorString);
  
    UI::End();
}

void LoadLibrary() {
    auto dll = Import::GetLibrary("SyncEdit.dll");

    if (dll is null) {
        g_errorString = "failed to load library: 'SyncEdit.dll'";

        return;
    }

    auto lastErrorString = dll.GetFunction("LastErrorString");

    if (lastErrorString is null) {
        g_errorString = "failed to load library function: 'LastErrorString' in 'SyncEdit.dll'";

        return;
    }

    auto join = dll.GetFunction("Join");

    if (join is null) {
        g_errorString = "failed to load library function: 'Join' in 'SyncEdit.dll'";

        return;
    }

    @g_library = Library(dll, lastErrorString, join);
}

class Library {
    private Import::Library@ dll;
    private Import::Function@ lastErrorString;
    private Import::Function@ join;

    Library(Import::Library@ dll, Import::Function@ lastErrorString, Import::Function@ join) {
        @this.dll = dll;
        @this.lastErrorString = lastErrorString;
        @this.join = join;
    }

    string LastErrorString() {
        return this.lastErrorString.CallString();
    }

    bool Join(const string&in host, uint16 port) {
        return this.join.CallBool(host, port);
    }
}
