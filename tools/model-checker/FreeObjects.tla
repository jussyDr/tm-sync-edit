---- MODULE FreeObjects ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxChannelSize, MaxValue, MaxNumValues
RECURSIVE SeqContains(_, _), SeqRemove(_, _), SeqCount(_, _), Broadcast(_, _, _, _)

SeqContains(seq, value) ==
    IF seq = <<>> THEN 
        FALSE
    ELSE
        IF Head(seq) = value THEN
            TRUE
        ELSE
            SeqContains(Tail(seq), value)

SeqRemove(seq, value) ==
    IF seq = <<>> THEN 
        <<>>
    ELSE
        IF Head(seq) = value THEN
            Tail(seq)
        ELSE
            SeqRemove(Tail(seq), value)

SeqCount(seq, value) ==
    IF seq = <<>> THEN
        0
    ELSE
        IF Head(seq) = value THEN
            1 + SeqCount(Tail(seq), value)
        ELSE 
            SeqCount(Tail(seq), value)

Broadcast(channels, command, exclude, i) ==
    IF channels = <<>> THEN
        <<>>
    ELSE
        IF i # exclude THEN
            <<Append(Head(channels), command)>> \o Broadcast(Tail(channels), command, exclude, i + 1)
        ELSE 
            <<Head(channels)>> \o Broadcast(Tail(channels), command, exclude, i + 1)

(* --algorithm free_objects

variables
    states = Append([client \in Clients |-> <<>>], <<>>);
    in_channels = [client \in Clients |-> <<>>];
    out_channels = [client \in Clients |-> <<>>];

define
    ChannelSizeConstraint == 
        \A client \in Clients: 
            Len(in_channels[client]) <= MaxChannelSize /\ Len(out_channels[client]) <= MaxChannelSize

    MaxNumValuesConstraint ==
        \A i \in 1..Len(states):
            Len(states[i]) <= MaxNumValues

    StatesEqualInvariant == 
        IF \A client \in Clients:
            Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
        THEN
            \A i \in 1..Len(states):
                \A j \in 1..Len(states):
                    \A value \in 1..MaxValue:
                        SeqCount(states[i], value) = SeqCount(states[j], value)
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
                        states[1] := Append(states[1], Head(in_channels[client]));
                        await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                        out_channels := Broadcast(out_channels, Head(in_channels[client]), client, 1);
                    else
                        if SeqContains(states[1], -Head(in_channels[client])) then
                            states[1] := SeqRemove(states[1], -Head(in_channels[client]));
                            await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                            out_channels := Broadcast(out_channels, Head(in_channels[client]), client, 1);
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
                        states[1 + self] := Append(states[1 + self], value);
                    end with;
                or
                    with value \in 1..MaxValue do
                        if SeqContains(states[1 + self], value) then
                            in_channels[self] := Append(in_channels[self], -value);
                            states[1 + self] := SeqRemove(states[1 + self], value);
                        end if;
                    end with;
                end either; 
            or
                await Len(out_channels[self]) > 0;
                if Head(out_channels[self]) > 0 then
                    states[1 + self] := Append(states[1 + self], Head(out_channels[self]));
                else
                    states[1 + self] := SeqRemove(states[1 + self], -Head(out_channels[self]));
                end if;
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "c3f737ee" /\ chksum(tla) = "ddfa0e68")
\* Label Run of process Server at line 73 col 9 changed to Run_
VARIABLES states, in_channels, out_channels

(* define statement *)
ChannelSizeConstraint ==
    \A client \in Clients:
        Len(in_channels[client]) <= MaxChannelSize /\ Len(out_channels[client]) <= MaxChannelSize

MaxNumValuesConstraint ==
    \A i \in 1..Len(states):
        Len(states[i]) <= MaxNumValues

StatesEqualInvariant ==
    IF \A client \in Clients:
        Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
    THEN
        \A i \in 1..Len(states):
            \A j \in 1..Len(states):
                \A value \in 1..MaxValue:
                    SeqCount(states[i], value) = SeqCount(states[j], value)
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
                             THEN /\ states' = [states EXCEPT ![1] = Append(states[1], Head(in_channels[client]))]
                                  /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                  /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]), client, 1)
                             ELSE /\ IF SeqContains(states[1], -Head(in_channels[client]))
                                        THEN /\ states' = [states EXCEPT ![1] = SeqRemove(states[1], -Head(in_channels[client]))]
                                             /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                             /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]), client, 1)
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                       /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                  ELSE /\ TRUE
                       /\ UNCHANGED << states, in_channels, out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < MaxChannelSize
                   /\ \/ /\ \E value \in 1..MaxValue:
                              /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], value)]
                              /\ states' = [states EXCEPT ![1 + self] = Append(states[1 + self], value)]
                      \/ /\ \E value \in 1..MaxValue:
                              IF SeqContains(states[1 + self], value)
                                 THEN /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], -value)]
                                      /\ states' = [states EXCEPT ![1 + self] = SeqRemove(states[1 + self], value)]
                                 ELSE /\ TRUE
                                      /\ UNCHANGED << states, in_channels >>
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ IF Head(out_channels[self]) > 0
                         THEN /\ states' = [states EXCEPT ![1 + self] = Append(states[1 + self], Head(out_channels[self]))]
                         ELSE /\ states' = [states EXCEPT ![1 + self] = SeqRemove(states[1 + self], -Head(out_channels[self]))]
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
