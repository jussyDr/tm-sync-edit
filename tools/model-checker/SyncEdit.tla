---- MODULE SyncEdit ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxChannelSize, MaxValue
RECURSIVE Broadcast(_, _, _, _)

Broadcast(channels, command, exclude, i) ==
    IF channels = <<>> THEN
        <<>>
    ELSE
        IF i # exclude THEN
            <<Append(Head(channels), command)>> \o Broadcast(Tail(channels), command, exclude, i + 1)
        ELSE
            <<Head(channels)>> \o Broadcast(Tail(channels), command, exclude, i + 1)

(* --algorithm sync_edit

variables
    states = Append([client \in Clients |-> 0], 0);
    in_channels = [client \in Clients |-> <<>>];
    out_channels = [client \in Clients |-> <<>>];

define
    ChannelSizeConstraint == 
        \A client \in Clients: 
            Len(in_channels[client]) <= MaxChannelSize /\ Len(out_channels[client]) <= MaxChannelSize

    StatesEqualInvariant == 
        IF \A client \in Clients:
            Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
        THEN
            \A i \in 1..Len(states):
                \A j \in 1..Len(states):
                    states[i] = states[j]
        ELSE
            TRUE
end define;

process Server = 1
begin
    Run:
        while TRUE do
            await \E client \in Clients: Len(in_channels[client]) > 0;
            with client \in Clients do
                if Len(in_channels[client]) > 0 then
                    states[1] := Head(in_channels[client]);
                    await \A out_client \in Clients: (out_client # client) => (Len(out_channels[out_client]) < MaxChannelSize);
                    out_channels := Broadcast(out_channels, Head(in_channels[client]), client, 1);
                    in_channels[client] := Tail(in_channels[client]);
                end if;
            end with;
        end while;
end process

process Client \in Clients
begin
    Run:
        while TRUE do
            either
                await Len(in_channels[self]) < MaxChannelSize;
                either
                    with value \in 1..MaxValue do 
                        in_channels[self] := Append(in_channels[self], value);
                        states[self + 1] := value;
                    end with;
                or
                    if states[self + 1] > 0 then
                        in_channels[self] := Append(in_channels[self], 0);
                        states[self + 1] := 0;
                    end if;
                end either;
            or
                await Len(out_channels[self]) > 0;
                states[self + 1] := Head(out_channels[self]);
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "d52d08ff" /\ chksum(tla) = "517534fe")
\* Label Run of process Server at line 41 col 9 changed to Run_
VARIABLES states, in_channels, out_channels

(* define statement *)
ChannelSizeConstraint ==
    \A client \in Clients:
        Len(in_channels[client]) <= MaxChannelSize /\ Len(out_channels[client]) <= MaxChannelSize

BlocksEqualInvariant ==
    IF \A client \in Clients:
        Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
    THEN
        \A i \in 1..Len(states):
            \A j \in 1..Len(states):
                states[i] = states[j]
    ELSE
        TRUE


vars == << states, in_channels, out_channels >>

ProcSet == {1} \cup (Clients)

Init == (* Global variables *)
        /\ states = Append([client \in Clients |-> 0], 0)
        /\ in_channels = [client \in Clients |-> <<>>]
        /\ out_channels = [client \in Clients |-> <<>>]

Server == /\ \E client \in Clients: Len(in_channels[client]) > 0
          /\ \E client \in Clients:
               IF Len(in_channels[client]) > 0
                  THEN /\ states' = [states EXCEPT ![1] = Head(in_channels[client])]
                       /\ \A out_client \in Clients: (out_client # client) => (Len(out_channels[out_client]) < MaxChannelSize)
                       /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]), client, 1)
                       /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                  ELSE /\ TRUE
                       /\ UNCHANGED << states, in_channels, out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < MaxChannelSize
                   /\ \/ /\ \E value \in 1..MaxValue:
                              /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], value)]
                              /\ states' = [states EXCEPT ![self + 1] = value]
                      \/ /\ IF states[self + 1] > 0
                               THEN /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], 0)]
                                    /\ states' = [states EXCEPT ![self + 1] = 0]
                               ELSE /\ TRUE
                                    /\ UNCHANGED << states, in_channels >>
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ states' = [states EXCEPT ![self + 1] = Head(out_channels[self])]
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
