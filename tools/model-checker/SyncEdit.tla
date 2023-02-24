---- MODULE SyncEdit ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxChannelSize, NumValues, MaxValue
RECURSIVE Broadcast(_, _)

Broadcast(channels, command) ==
    IF channels = <<>> THEN
        <<>>
    ELSE
        <<Append(Head(channels), command)>> \o Broadcast(Tail(channels), command)

(* --algorithm sync_edit

variables
    states = Append([client \in Clients |-> [index \in 1..NumValues |-> 0]], [index \in 1..NumValues |-> 0]);
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
                    if Head(in_channels[client]).v > 0 then
                        if states[1][Head(in_channels[client]).i] = 0 then
                            states[1][Head(in_channels[client]).i] := Head(in_channels[client]).v;
                            await \A out_client \in Clients: (Len(out_channels[out_client]) < MaxChannelSize);
                            out_channels := Broadcast(out_channels, [i |-> Head(in_channels[client]).i, v |-> Head(in_channels[client]).v]);
                        end if;
                    else
                        if states[1][Head(in_channels[client]).i] > 0 then
                            states[1][Head(in_channels[client]).i] := 0;
                            await \A out_client \in Clients: (Len(out_channels[out_client]) < MaxChannelSize);
                            out_channels := Broadcast(out_channels, [i |-> Head(in_channels[client]).i, v |-> Head(in_channels[client]).v]);
                        end if;
                    end if; 
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
                with index \in 1..NumValues do
                    if states[1 + self][index] = 0 then
                        with value \in 1..MaxValue do 
                            in_channels[self] := Append(in_channels[self], [i |-> index, v |-> value]);
                            states[1 + self][index] := value;
                        end with;
                    else
                        in_channels[self] := Append(in_channels[self], [i |-> index, v |-> 0]);
                        states[1 + self][index] := 0;
                    end if;
                end with;
            or
                await Len(out_channels[self]) > 0;
                states[1 + self][Head(out_channels[self]).i] := Head(out_channels[self]).v;
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "5b9efc1b" /\ chksum(tla) = "42762477")
\* Label Run of process Server at line 38 col 9 changed to Run_
VARIABLES states, in_channels, out_channels

(* define statement *)
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


vars == << states, in_channels, out_channels >>

ProcSet == {1} \cup (Clients)

Init == (* Global variables *)
        /\ states = Append([client \in Clients |-> [index \in 1..NumValues |-> 0]], [index \in 1..NumValues |-> 0])
        /\ in_channels = [client \in Clients |-> <<>>]
        /\ out_channels = [client \in Clients |-> <<>>]

Server == /\ \E client \in Clients: Len(in_channels[client]) > 0
          /\ \E client \in Clients:
               IF Len(in_channels[client]) > 0
                  THEN /\ IF Head(in_channels[client]).v > 0
                             THEN /\ IF states[1][Head(in_channels[client]).i] = 0
                                        THEN /\ states' = [states EXCEPT ![1][Head(in_channels[client]).i] = Head(in_channels[client]).v]
                                             /\ \A out_client \in Clients: (Len(out_channels[out_client]) < MaxChannelSize)
                                             /\ out_channels' = Broadcast(out_channels, [i |-> Head(in_channels[client]).i, v |-> Head(in_channels[client]).v])
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                             ELSE /\ IF states[1][Head(in_channels[client]).i] > 0
                                        THEN /\ states' = [states EXCEPT ![1][Head(in_channels[client]).i] = 0]
                                             /\ \A out_client \in Clients: (Len(out_channels[out_client]) < MaxChannelSize)
                                             /\ out_channels' = Broadcast(out_channels, [i |-> Head(in_channels[client]).i, v |-> Head(in_channels[client]).v])
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                       /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                  ELSE /\ TRUE
                       /\ UNCHANGED << states, in_channels, out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < MaxChannelSize
                   /\ \E index \in 1..NumValues:
                        IF states[1 + self][index] = 0
                           THEN /\ \E value \in 1..MaxValue:
                                     /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], [i |-> index, v |-> value])]
                                     /\ states' = [states EXCEPT ![1 + self][index] = value]
                           ELSE /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], [i |-> index, v |-> 0])]
                                /\ states' = [states EXCEPT ![1 + self][index] = 0]
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ states' = [states EXCEPT ![1 + self][Head(out_channels[self]).i] = Head(out_channels[self]).v]
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
