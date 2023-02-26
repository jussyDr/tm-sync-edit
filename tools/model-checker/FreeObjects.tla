---- MODULE FreeObjects ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxChannelSize, MaxValue, MaxDuplicateValues
RECURSIVE MultisetInsert(_, _), MultisetRemove(_, _), MultisetCount(_, _), MultisetSetCount(_, _, _), Broadcast(_, _)

MultisetInsert(set, value) ==
    IF set = <<>> THEN
        <<<<value, 1>>>>
    ELSE
        IF Head(set)[1] = value THEN
            Append(Tail(set), <<value, Head(set)[2] + 1>>)
        ELSE
            Append(MultisetInsert(Tail(set), value), Head(set))

MultisetRemove(set, value) ==
    IF set = <<>> THEN
        <<>>
    ELSE
        IF Head(set)[1] = value THEN
            IF Head(set)[2] = 1 THEN
                Tail(set)
            ELSE
                Append(Tail(set), <<value, Head(set)[2] - 1>>)
        ELSE
            Append(MultisetRemove(Tail(set), value), Head(set))

MultisetCount(set, value) ==
    IF set = <<>> THEN
        0
    ELSE
        IF Head(set)[1] = value THEN
            Head(set)[2]
        ELSE
            MultisetCount(Tail(set), value)

MultisetSetCount(set, value, count) ==
    IF set = <<>> THEN
        <<<<value, count>>>>
    ELSE
        IF Head(set)[1] = value THEN
            Append(Tail(set), <<value, count>>)
        ELSE
            Append(MultisetSetCount(Tail(set), value, count), Head(set))

Broadcast(channels, message) ==
    IF channels = <<>> THEN
        <<>>
    ELSE
        <<Append(Head(channels), message)>> \o Broadcast(Tail(channels), message)

(* --algorithm free_objects

variables
    states = Append([client \in Clients |-> <<>>], <<>>);
    in_channels = [client \in Clients |-> <<>>];
    out_channels = [client \in Clients |-> <<>>];

define
    ChannelSizeConstraint == 
        \A client \in Clients: 
            Len(in_channels[client]) <= MaxChannelSize /\ Len(out_channels[client]) <= MaxChannelSize

    DuplicateValuesConstraint ==
        \A i \in 1..Len(states):
            \A j \in 1..Len(states[i]):
                states[i][j][2] <= MaxDuplicateValues

    StatesEqualInvariant == 
        IF \A client \in Clients:
            Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
        THEN
            \A i \in 1..Len(states):
                \A j \in 1..Len(states):
                    \A value \in 1..MaxValue:
                        MultisetCount(states[i], value) = MultisetCount(states[j], value)
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
                    if Head(in_channels[client]) > 0 then
                        states[1] := MultisetInsert(states[1], Head(in_channels[client]));
                        await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                        out_channels := Broadcast(out_channels, <<Head(in_channels[client]), MultisetCount(states[1], Head(in_channels[client]))>>);
                    else
                        if MultisetCount(states[1], -Head(in_channels[client])) > 0 then
                            states[1] := MultisetRemove(states[1], -Head(in_channels[client]));
                            await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                            out_channels := Broadcast(out_channels, <<-Head(in_channels[client]), MultisetCount(states[1], -Head(in_channels[client]))>>);
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
                either
                    with value \in 1..MaxValue do 
                        in_channels[self] := Append(in_channels[self], value);
                        states[1 + self] := MultisetInsert(states[1 + self], value);
                    end with;
                or
                    with value \in 1..MaxValue do
                        if MultisetCount(states[1 + self], value) > 0 then
                            in_channels[self] := Append(in_channels[self], -value);
                            states[1 + self] := MultisetRemove(states[1 + self], value);
                        end if;
                    end with;
                end either; 
            or
                await Len(out_channels[self]) > 0;
                states[1 + self] := MultisetSetCount(states[1 + self], Head(out_channels[self])[1], Head(out_channels[self])[2]);
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "fea3f636" /\ chksum(tla) = "2b14196a")
\* Label Run of process Server at line 83 col 9 changed to Run_
VARIABLES states, in_channels, out_channels

(* define statement *)
ChannelSizeConstraint ==
    \A client \in Clients:
        Len(in_channels[client]) <= MaxChannelSize /\ Len(out_channels[client]) <= MaxChannelSize

DuplicateValuesConstraint ==
    \A i \in 1..Len(states):
        \A j \in 1..Len(states[i]):
            states[i][j][2] <= MaxDuplicateValues

StatesEqualInvariant ==
    IF \A client \in Clients:
        Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
    THEN
        \A i \in 1..Len(states):
            \A j \in 1..Len(states):
                \A value \in 1..MaxValue:
                    MultisetCount(states[i], value) = MultisetCount(states[j], value)
    ELSE
        TRUE


vars == << states, in_channels, out_channels >>

ProcSet == {1} \cup (Clients)

Init == (* Global variables *)
        /\ states = Append([client \in Clients |-> <<>>], <<>>)
        /\ in_channels = [client \in Clients |-> <<>>]
        /\ out_channels = [client \in Clients |-> <<>>]

Server == /\ \E client \in Clients: Len(in_channels[client]) > 0
          /\ \E client \in Clients:
               IF Len(in_channels[client]) > 0
                  THEN /\ IF Head(in_channels[client]) > 0
                             THEN /\ states' = [states EXCEPT ![1] = MultisetInsert(states[1], Head(in_channels[client]))]
                                  /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                  /\ out_channels' = Broadcast(out_channels, <<Head(in_channels[client]), MultisetCount(states'[1], Head(in_channels[client]))>>)
                             ELSE /\ IF MultisetCount(states[1], -Head(in_channels[client])) > 0
                                        THEN /\ states' = [states EXCEPT ![1] = MultisetRemove(states[1], -Head(in_channels[client]))]
                                             /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                             /\ out_channels' = Broadcast(out_channels, <<-Head(in_channels[client]), MultisetCount(states'[1], -Head(in_channels[client]))>>)
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                       /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                  ELSE /\ TRUE
                       /\ UNCHANGED << states, in_channels, out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < MaxChannelSize
                   /\ \/ /\ \E value \in 1..MaxValue:
                              /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], value)]
                              /\ states' = [states EXCEPT ![1 + self] = MultisetInsert(states[1 + self], value)]
                      \/ /\ \E value \in 1..MaxValue:
                              IF MultisetCount(states[1 + self], value) > 0
                                 THEN /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], -value)]
                                      /\ states' = [states EXCEPT ![1 + self] = MultisetRemove(states[1 + self], value)]
                                 ELSE /\ TRUE
                                      /\ UNCHANGED << states, in_channels >>
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ states' = [states EXCEPT ![1 + self] = MultisetSetCount(states[1 + self], Head(out_channels[self])[1], Head(out_channels[self])[2])]
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
