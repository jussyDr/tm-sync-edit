---- MODULE Blocks ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxChannelSize, NumValues, MaxValue
RECURSIVE Broadcast(_, _)

Broadcast(channels, message) ==
    IF channels = <<>> THEN
        <<>>
    ELSE
        <<Append(Head(channels), message)>> \o Broadcast(Tail(channels), message)

(* --algorithm blocks

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
            (* wait for a message from any of the clients *)
            await \E client \in Clients: Len(in_channels[client]) > 0;

            with client \in Clients do
                if Len(in_channels[client]) > 0 then
                    if Head(in_channels[client]).v > 0 
                    then (* received a message to place a block *)
                        if states[1][Head(in_channels[client]).i] = 0 
                        then (* the coordinate is not already occupied by another block *)
                            either
                                states[1][Head(in_channels[client]).i] := Head(in_channels[client]).v;
                                await \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize;
                                out_channels := Broadcast(out_channels, Head(in_channels[client]));
                            or (* case in which the object could not be placed (e.g. if it is out of bounds or has a unknown model variant) *)
                                await Len(out_channels[client]) < MaxChannelSize;
                                out_channels[client] := Append(out_channels[client], [i |-> Head(in_channels[client]).i, v |-> -Head(in_channels[client]).v]);
                            end either;
                        end if;
                    else (* received a message to remove a block *)
                        if states[1][Head(in_channels[client]).i] = -Head(in_channels[client]).v 
                        then (* the block we want to remove actually exists *)
                            states[1][Head(in_channels[client]).i] := 0;
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
            either (* send a message to the server *)
                await Len(in_channels[self]) < MaxChannelSize;

                with index \in 1..NumValues do
                    if states[1 + self][index] = 0 
                    then (* send a message to place a block *)
                        with value \in 1..MaxValue do 
                            in_channels[self] := Append(in_channels[self], [i |-> index, v |-> value]);
                            states[1 + self][index] := value;
                        end with;
                    else (* send a message to remove a block *)
                        in_channels[self] := Append(in_channels[self], [i |-> index, v |-> -states[1 + self][index]]);
                        states[1 + self][index] := 0;
                    end if;
                end with;
            or (* receive a message from the server *)
                await Len(out_channels[self]) > 0;

                if Head(out_channels[self]).v > 0 then
                    states[1 + self][Head(out_channels[self]).i] := Head(out_channels[self]).v;
                else
                    if states[1 + self][Head(out_channels[self]).i] = -Head(out_channels[self]).v then
                        states[1 + self][Head(out_channels[self]).i] := 0;
                    end if;
                end if;

                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "be0f193e" /\ chksum(tla) = "68dcbf9a")
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
                                        THEN /\ \/ /\ states' = [states EXCEPT ![1][Head(in_channels[client]).i] = Head(in_channels[client]).v]
                                                   /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                                   /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]))
                                                \/ /\ Len(out_channels[client]) < MaxChannelSize
                                                   /\ out_channels' = [out_channels EXCEPT ![client] = Append(out_channels[client], [i |-> Head(in_channels[client]).i, v |-> -Head(in_channels[client]).v])]
                                                   /\ UNCHANGED states
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << states, 
                                                             out_channels >>
                             ELSE /\ IF states[1][Head(in_channels[client]).i] = -Head(in_channels[client]).v
                                        THEN /\ states' = [states EXCEPT ![1][Head(in_channels[client]).i] = 0]
                                             /\ \A out_client \in Clients: Len(out_channels[out_client]) < MaxChannelSize
                                             /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]))
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
                           ELSE /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], [i |-> index, v |-> -states[1 + self][index]])]
                                /\ states' = [states EXCEPT ![1 + self][index] = 0]
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ IF Head(out_channels[self]).v > 0
                         THEN /\ states' = [states EXCEPT ![1 + self][Head(out_channels[self]).i] = Head(out_channels[self]).v]
                         ELSE /\ IF states[1 + self][Head(out_channels[self]).i] = -Head(out_channels[self]).v
                                    THEN /\ states' = [states EXCEPT ![1 + self][Head(out_channels[self]).i] = 0]
                                    ELSE /\ TRUE
                                         /\ UNCHANGED states
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
