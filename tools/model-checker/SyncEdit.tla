---- MODULE SyncEdit ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, MaxTime, MaxChannelSize, MaxValue
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
    times = Append([client \in Clients |-> 0], 0);
    in_channels = [client \in Clients |-> <<>>];
    out_channels = [client \in Clients |-> <<>>];

define
    TimeConstraint ==
        times[1] <= MaxTime

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
                    if Head(in_channels[client]).t = times[1] then
                        if Head(in_channels[client]).v > 0 then
                            if states[1] = 0 then
                                states[1] := Head(in_channels[client]).v;
                                times[1] := times[1] + 1;
                                await \A out_client \in Clients: (out_client # client) => (Len(out_channels[out_client]) < MaxChannelSize);
                                out_channels := Broadcast(out_channels, [v |-> Head(in_channels[client]).v, t |-> times[1]], client, 1);
                                in_channels[client] := Tail(in_channels[client]);
                            end if;
                        else
                            if states[1] > 0 then
                                states[1] := 0;
                                times[1] := times[1] + 1;
                                await \A out_client \in Clients: (out_client # client) => (Len(out_channels[out_client]) < MaxChannelSize);
                                out_channels := Broadcast(out_channels, [v |-> Head(in_channels[client]).v, t |-> times[1]], client, 1);
                                in_channels[client] := Tail(in_channels[client]);
                            end if;
                        end if; 
                    end if;
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
                if states[1 + self] = 0 then
                    with value \in 1..MaxValue do 
                        in_channels[self] := Append(in_channels[self], [v |-> value, t |-> times[1 + self]]);
                        states[1 + self] := value;
                    end with;
                else
                    in_channels[self] := Append(in_channels[self], [v |-> 0, t |-> times[1 + self]]);
                    states[1 + self] := 0;
                end if;
            or
                await Len(out_channels[self]) > 0;
                states[1 + self] := Head(out_channels[self]).v;
                times[1 + self] := Head(out_channels[self]).t;
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "4ecabcc6" /\ chksum(tla) = "e61a743f")
\* Label Run of process Server at line 45 col 9 changed to Run_
VARIABLES states, times, in_channels, out_channels

(* define statement *)
TimeConstraint ==
    times[1] <= MaxTime

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


vars == << states, times, in_channels, out_channels >>

ProcSet == {1} \cup (Clients)

Init == (* Global variables *)
        /\ states = Append([client \in Clients |-> 0], 0)
        /\ times = Append([client \in Clients |-> 0], 0)
        /\ in_channels = [client \in Clients |-> <<>>]
        /\ out_channels = [client \in Clients |-> <<>>]

Server == /\ \E client \in Clients: Len(in_channels[client]) > 0
          /\ \E client \in Clients:
               IF Len(in_channels[client]) > 0
                  THEN /\ IF Head(in_channels[client]).t = times[1]
                             THEN /\ IF Head(in_channels[client]).v > 0
                                        THEN /\ IF states[1] = 0
                                                   THEN /\ states' = [states EXCEPT ![1] = Head(in_channels[client]).v]
                                                        /\ times' = [times EXCEPT ![1] = times[1] + 1]
                                                        /\ \A out_client \in Clients: (out_client # client) => (Len(out_channels[out_client]) < MaxChannelSize)
                                                        /\ out_channels' = Broadcast(out_channels, [v |-> Head(in_channels[client]).v, t |-> times'[1]], client, 1)
                                                        /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                                                   ELSE /\ TRUE
                                                        /\ UNCHANGED << states, 
                                                                        times, 
                                                                        in_channels, 
                                                                        out_channels >>
                                        ELSE /\ IF states[1] > 0
                                                   THEN /\ states' = [states EXCEPT ![1] = 0]
                                                        /\ times' = [times EXCEPT ![1] = times[1] + 1]
                                                        /\ \A out_client \in Clients: (out_client # client) => (Len(out_channels[out_client]) < MaxChannelSize)
                                                        /\ out_channels' = Broadcast(out_channels, [v |-> Head(in_channels[client]).v, t |-> times'[1]], client, 1)
                                                        /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                                                   ELSE /\ TRUE
                                                        /\ UNCHANGED << states, 
                                                                        times, 
                                                                        in_channels, 
                                                                        out_channels >>
                             ELSE /\ TRUE
                                  /\ UNCHANGED << states, times, in_channels, 
                                                  out_channels >>
                  ELSE /\ TRUE
                       /\ UNCHANGED << states, times, in_channels, 
                                       out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < MaxChannelSize
                   /\ IF states[1 + self] = 0
                         THEN /\ \E value \in 1..MaxValue:
                                   /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], [v |-> value, t |-> times[1 + self]])]
                                   /\ states' = [states EXCEPT ![1 + self] = value]
                         ELSE /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], [v |-> 0, t |-> times[1 + self]])]
                              /\ states' = [states EXCEPT ![1 + self] = 0]
                   /\ UNCHANGED <<times, out_channels>>
                \/ /\ Len(out_channels[self]) > 0
                   /\ states' = [states EXCEPT ![1 + self] = Head(out_channels[self]).v]
                   /\ times' = [times EXCEPT ![1 + self] = Head(out_channels[self]).t]
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
