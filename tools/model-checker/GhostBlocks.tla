---- MODULE GhostBlocks ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxChannelSize, MaxValue
RECURSIVE Broadcast(_, _)

Broadcast(channels, command) ==
    IF channels = <<>> THEN
        <<>>
    ELSE
        <<Append(Head(channels), command)>> \o Broadcast(Tail(channels), command)

(* --algorithm ghost_blocks

variables
    states = Append([client \in Clients |-> {}], {});
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
                    if Head(in_channels[client]) > 0 then
                        if Head(in_channels[client]) \notin states[1] then
                            states[1] := states[1] \union {Head(in_channels[client])};
                            await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                            out_channels := Broadcast(out_channels, Head(in_channels[client]));
                        end if;
                    else
                        if -Head(in_channels[client]) \in states[1] then
                            states[1] := states[1] \ {-Head(in_channels[client])};
                            await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                            out_channels := Broadcast(out_channels, Head(in_channels[client]));
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
                    with value \in (1..MaxValue \ states[1 + self]) do 
                        in_channels[self] := Append(in_channels[self], value);
                        states[1 + self] := states[1 + self] \union {value};
                    end with;
                or
                    with value \in states[1 + self] do
                        in_channels[self] := Append(in_channels[self], -value);
                        states[1 + self] := states[1 + self] \ {value};
                    end with;
                end either; 
            or
                await Len(out_channels[self]) > 0;
                if Head(out_channels[self]) > 0 then
                    states[1 + self] := states[1 + self] \union {Head(out_channels[self])};
                else
                    states[1 + self] := states[1 + self] \ {-Head(out_channels[self])};
                end if;
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "b04ac8d" /\ chksum(tla) = "c32469cb")
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
        /\ states = Append([client \in Clients |-> {}], {})
        /\ in_channels = [client \in Clients |-> <<>>]
        /\ out_channels = [client \in Clients |-> <<>>]

Server == /\ \E client \in Clients: Len(in_channels[client]) > 0
          /\ \E client \in Clients:
               IF Len(in_channels[client]) > 0
                  THEN /\ IF Head(in_channels[client]) > 0
                             THEN /\ IF Head(in_channels[client]) \notin states[1]
                                        THEN /\ states' = [states EXCEPT ![1] = states[1] \union {Head(in_channels[client])}]
                                             /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                             /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]))
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                             ELSE /\ IF -Head(in_channels[client]) \in states[1]
                                        THEN /\ states' = [states EXCEPT ![1] = states[1] \ {-Head(in_channels[client])}]
                                             /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                             /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]))
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                       /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                  ELSE /\ TRUE
                       /\ UNCHANGED << states, in_channels, out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < MaxChannelSize
                   /\ \/ /\ \E value \in (1..MaxValue \ states[1 + self]):
                              /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], value)]
                              /\ states' = [states EXCEPT ![1 + self] = states[1 + self] \union {value}]
                      \/ /\ \E value \in states[1 + self]:
                              /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], -value)]
                              /\ states' = [states EXCEPT ![1 + self] = states[1 + self] \ {value}]
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ IF Head(out_channels[self]) > 0
                         THEN /\ states' = [states EXCEPT ![1 + self] = states[1 + self] \union {Head(out_channels[self])}]
                         ELSE /\ states' = [states EXCEPT ![1 + self] = states[1 + self] \ {-Head(out_channels[self])}]
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
