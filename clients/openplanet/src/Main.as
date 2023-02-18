const string c_title = Icons::Syncthing + " Sync Edit";

bool g_interfaceVisible;
string g_status = "disconnected";

bool g_disconnected = true;
bool g_canceled = false;
Net::Socket@ g_socket;

uint g_numBlocksPlaced = 0;
uint g_numItemsPlaced = 0;

void Main() {

}

void RenderInterface() {
    if (UI::Begin(c_title)) {
        if (g_disconnected) {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);
            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

            if (UI::Button("Join")) {
                startnew(Loop);
            }
        } else {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                g_canceled = true;
            }
        }

        UI::Text(g_status);

        UI::End();
    }
}

void RenderMenu() {
    if (UI::MenuItem(c_title, "", g_interfaceVisible)) {
        g_interfaceVisible = !g_interfaceVisible;
    }
}

void OnDisabled() {

}

void OnEnabled() {
    
}

void OnDestroyed() {
    if (g_socket !is null) {
        g_socket.Close();
        @g_socket = null;
    }

    g_disconnected = true;
}

void OnPlaceBlock() {
    g_numBlocksPlaced++;
}

void OnRemoveBlock(CGameCtnBlock@ rdx) {

}

void OnPlaceItem() {
    g_numItemsPlaced++;
}

void OnRemoveItem(CGameCtnAnchoredObject@ rdx) {

}

void Error(const string&in message) {
    OnDestroyed();
    g_status = message;
}

void Loop() {
    g_disconnected = false;

    if (Setting_Port.Length == 0) {
        Error("invalid port");
        return;
    }

    auto port = Text::ParseUInt(Setting_Port);
    
    if (port > 65535) {
        Error("invalid port");
        return;
    }

    @g_socket = Net::Socket();

    if (!g_socket.Connect(Setting_Host, port)) {
        Error("invalid host");
        return;
    }

    g_status = "connecting";

    auto connectTime = Time::Now;

    while (!g_socket.CanWrite()) {
        yield();

        if (Time::Now >= connectTime + Setting_ConnectTimeout) {
            Error("timed out");
            return;
        }

        if (g_canceled) {
            g_canceled = false;
            Error("canceled");
            return;
        }
    }

    while (true) {
        while (!g_socket.CanRead()) {
            yield();

            if (g_numBlocksPlaced > 0) {
                g_numBlocksPlaced = 0;
            }

            if (g_numItemsPlaced > 0) {
                g_numItemsPlaced = 0;
            }
        }

        auto available = g_socket.Available();

        if (available == 0) {
            Error("server disconnected");
            return;
        }

        while (available > 0) {
            if (available < 4) {
                Error("disconnected");
                return
            }

            uint frameLength = g_socket.ReadUint32();
            available -= 4;

            if (available < frameLength) {
                Error("disconnected");
                return
            }

            auto json = g_socket.ReadRaw(frameLength);
            available -= frameLength;

            auto commandValue = Json::Parse(json);

            if (commandValue.HasKey("PlaceBlock")) {
                print("received: PlaceBlock");
            } else if (commandValue.HasKey("RemoveBlock")) {
                print("received: RemoveBlock");
            } else if (commandValue.HasKey("PlaceFreeBlock")) {
                print("received: PlaceFreeBlock");
            } else if (commandValue.HasKey("RemoveFreeBlock")) {
                print("received: RemoveFreeBlock");
            } else if (commandValue.HasKey("PlaceItem")) {
                print("received: PlaceItem");
            } else if (commandValue.HasKey("RemoveItem")) {
                print("received: RemoveItem");
            }
        }
    }
}
