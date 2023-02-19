const string c_title = Icons::Syncthing + " Sync Edit";

enum State {
    Disconnected,
    Connecting,
    Connected,
}

bool g_interfaceVisible;
string g_status;

State g_state;
bool g_stopped;
Net::Socket@ g_socket;

Dev::HookInfo@ g_hookPlaceBlock;
Dev::HookInfo@ g_hookRemoveBlock;
Dev::HookInfo@ g_hookPlaceItem;
Dev::HookInfo@ g_hookRemoveItem;

uint g_numBlocksPlaced;
uint g_numItemsPlaced;

void Main() {
    g_status = "Disconnected";
    g_state = State::Disconnected;
    g_stopped = false;
    g_numBlocksPlaced = 0;
    g_numItemsPlaced = 0;
}

void RenderInterface() {
    if (UI::Begin(c_title)) {
        if (g_state == State::Disconnected) {
            Setting_Host = UI::InputText("Host", Setting_Host, UI::InputTextFlags::CharsNoBlank);
            Setting_Port = UI::InputText("Port", Setting_Port, UI::InputTextFlags::CharsDecimal);

            if (UI::Button("Join")) {
                startnew(MainLoop);
            }
        } else if (g_state == State::Connecting) {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Cancel")) {
                g_stopped = true;
            }
        } else if (g_state == State::Connected) {
            UI::LabelText("Host", Setting_Host);
            UI::LabelText("Port", Setting_Port);

            if (UI::Button("Exit")) {
                g_stopped = true;
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
    if (g_hookPlaceBlock !is null) {
        Dev::Unhook(g_hookPlaceBlock);
        @g_hookPlaceBlock = null;
    }

    if (g_hookRemoveBlock !is null) {
        Dev::Unhook(g_hookRemoveBlock);
        @g_hookRemoveBlock = null;
    }

    if (g_hookPlaceItem !is null) {
        Dev::Unhook(g_hookPlaceItem);
        @g_hookPlaceItem = null;
    }

    if (g_hookRemoveItem !is null) {
        Dev::Unhook(g_hookRemoveItem);
        @g_hookRemoveItem = null;
    }

    if (g_socket !is null) {
        g_socket.Close();
        @g_socket = null;
    }

    g_state = State::Disconnected;
}

void OnPlaceBlock() {
    g_numBlocksPlaced++;
}

void OnRemoveBlock(CGameCtnBlock@ rdx) {
    if (g_numBlocksPlaced > 0) {
        g_numBlocksPlaced--;
    } else {
        print("removed block");
    }
}

void OnPlaceItem() {
    g_numItemsPlaced++;
}

void OnRemoveItem(CGameCtnAnchoredObject@ rdx) {
    print("removed item");
}

void Error(const string&in message) {
    OnDestroyed();
    g_status = message;
}

void MainLoop() {
    g_state = State::Connecting;

    if (Setting_Port.Length == 0) {
        Error("Invalid port");
        return;
    }

    auto port = Text::ParseUInt(Setting_Port);
    
    if (port > 65535) {
        Error("Invalid port");
        return;
    }

    @g_socket = Net::Socket();

    if (!g_socket.Connect(Setting_Host, port)) {
        Error("Invalid host");
        return;
    }

    g_status = "Connecting";

    auto connectTime = Time::Now;

    while (!g_socket.CanWrite()) {
        yield();

        if (Time::Now >= connectTime + Setting_ConnectTimeout) {
            Error("Timed out");
            return;
        }

        if (g_stopped) {
            g_stopped = false;
            Error("Canceled");
            return;
        }
    }

    g_status = "Connected";
    g_state = State::Connected;

    auto pfPlaceBlock = Dev::FindPattern("48 89 5c 24 10 48 89 74 24 20 4c 89 44 24 18 55 57 41 55");

    if (pfPlaceBlock == 0) {
        Error("Failed to find function 'PlaceBlock'");
        return;
    }

    auto pfRemoveBlock = Dev::FindPattern("48 89 5c 24 08 48 89 6c 24 10 48 89 74 24 18 57 48 83 ec 40 83 7c");

    if (pfRemoveBlock == 0) {
        Error("Failed to find function 'RemoveBlock'");
        return;
    }

    auto pfPlaceItem = Dev::FindPattern("48 89 5c 24 18 55 56 57 48 83 ec 40 49");

    if (pfPlaceItem == 0) {
        Error("Failed to find function 'PlaceItem'");
        return;
    }

    auto pfRemoveItem = Dev::FindPattern("48 89 5c 24 08 57 48 83 ec 30 48 8b fa 48 8b d9 48 85 d2 0f");

    if (pfRemoveItem == 0) {
        Error("Failed to find function 'RemoveItem'");
        return;
    }

    @g_hookPlaceBlock = Dev::Hook(pfPlaceBlock, 0, "OnPlaceBlock");

    if (g_hookPlaceBlock is null) {
        Error("Failed to hook function 'PlaceBlock'");
        return;
    }
    
    @g_hookRemoveBlock = Dev::Hook(pfRemoveBlock, 0, "OnRemoveBlock");

    if (g_hookRemoveBlock is null) {
        Error("Failed to hook function 'RemoveBlock'");
        return;
    }

    @g_hookPlaceItem = Dev::Hook(pfPlaceItem, 0, "OnPlaceItem");

    if (g_hookPlaceItem is null) {
        Error("Failed to hook function 'PlaceItem'");
        return;
    }

    @g_hookRemoveItem = Dev::Hook(pfRemoveItem, 0, "OnRemoveItem");

    if (g_hookRemoveItem is null) {
        Error("Failed to hook function 'RemoveItem'");
        return;
    }

    while (true) {
        auto editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

        if (editor is null) {
            Error("Exited editor");
            return;
        }

        while (!g_socket.CanRead()) {
            yield();

            if (g_stopped) {
                g_stopped = false;
                Error("Exited");
                return;
            }

            @editor = cast<CGameCtnEditorCommon>(GetApp().Editor);

            if (editor is null) {
                Error("Exited editor");
                return;
            }

            while (g_numBlocksPlaced > 0) {
                print("placed block");

                g_numBlocksPlaced--;
            }

            while (g_numItemsPlaced > 0) {
                print("placed item");

                g_numItemsPlaced--;
            }
        }

        uint available = g_socket.Available();

        if (available == 0) {
            Error("Server disconnected");
            return;
        }

        while (available > 0) {
            if (available < 4) {
                Error("Disconnected");
                return;
            }

            uint frameLength = g_socket.ReadUint32();
            available -= 4;

            if (available < frameLength) {
                Error("Disconnected");
                return;
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
