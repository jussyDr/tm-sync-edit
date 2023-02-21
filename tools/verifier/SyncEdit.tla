---- MODULE SyncEdit ----
EXTENDS Integers, FiniteSets, Sequences
CONSTANTS Clients, CountMax, ChannelSize
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
    counts = Append([client \in Clients |-> 0], 0);
    in_channels = [client \in Clients |-> <<>>];
    out_channels = [client \in Clients |-> <<>>];

define
    CountsConstraint ==
        \A i \in 1..Len(counts):
            counts[i] <= CountMax

    ChannelSizeConstraint == 
        \A client \in Clients: 
            Len(in_channels[client]) <= ChannelSize /\ Len(out_channels[client]) <= ChannelSize

    CountsNonNegativeInvariant ==
        \A i \in 1..Len(counts):
            counts[i] >= 0

    CountsEqualInvariant == 
        IF \A client \in Clients:
            Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
        THEN
            \A i \in 1..Len(counts):
                \A j \in 1..Len(counts):
                    counts[i] = counts[j]
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
                    if Head(in_channels[client]) = "Place" then
                        counts[1] := counts[1] + 1;
                    else 
                        if Head(in_channels[client]) = "Remove" then
                            if counts[1] > 0 then
                                counts[1] := counts[1] - 1;
                            end if;
                        end if;
                    end if;
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
                await Len(in_channels[self]) < ChannelSize;
                either
                    in_channels[self] := Append(in_channels[self], "Place");
                    counts[self + 1] := counts[self + 1] + 1;
                or
                    if counts[self + 1] > 0 then
                        in_channels[self] := Append(in_channels[self], "Remove");
                        counts[self + 1] := counts[self + 1] - 1;
                    end if;
                end either;
            or
                await Len(out_channels[self]) > 0;
                if Head(out_channels[self]) = "Place" then
                    counts[self + 1] := counts[self + 1] + 1;
                else 
                    if Head(out_channels[self]) = "Remove" then
                        counts[self + 1] := counts[self + 1] - 1;
                    end if;
                end if;
                out_channels[self] := Tail(out_channels[self]);
            end either;
        end while;
end process

end algorithm *)

\* BEGIN TRANSLATION (chksum(pcal) = "ad89b365" /\ chksum(tla) = "6cb0a1e5")
\* Label Run of process Server at line 52 col 9 changed to Run_
VARIABLES counts, in_channels, out_channels

(* define statement *)
CountsConstraint ==
    \A i \in 1..Len(counts):
        counts[i] <= CountMax

ChannelSizeConstraint ==
    \A client \in Clients:
        Len(in_channels[client]) <= ChannelSize /\ Len(out_channels[client]) <= ChannelSize

CountsNonNegativeInvariant ==
    \A i \in 1..Len(counts):
        counts[i] >= 0

CountsEqualInvariant ==
    IF \A client \in Clients:
        Len(in_channels[client]) = 0 /\ Len(out_channels[client]) = 0
    THEN
        \A i \in 1..Len(counts):
            \A j \in 1..Len(counts):
                counts[i] = counts[j]
    ELSE
        TRUE


vars == << counts, in_channels, out_channels >>

ProcSet == {1} \cup (Clients)

Init == (* Global variables *)
        /\ counts = Append([client \in Clients |-> 0], 0)
        /\ in_channels = [client \in Clients |-> <<>>]
        /\ out_channels = [client \in Clients |-> <<>>]

Server == /\ \E client \in Clients: Len(in_channels[client]) > 0
          /\ \E client \in Clients:
               IF Len(in_channels[client]) > 0
                  THEN /\ IF Head(in_channels[client]) = "Place"
                             THEN /\ counts' = [counts EXCEPT ![1] = counts[1] + 1]
                             ELSE /\ IF Head(in_channels[client]) = "Remove"
                                        THEN /\ IF counts[1] > 0
                                                   THEN /\ counts' = [counts EXCEPT ![1] = counts[1] - 1]
                                                   ELSE /\ TRUE
                                                        /\ UNCHANGED counts
                                        ELSE /\ TRUE
                                             /\ UNCHANGED counts
                       /\ out_channels' = Broadcast(out_channels, Head(in_channels[client]), client, 1)
                       /\ in_channels' = [in_channels EXCEPT ![client] = Tail(in_channels[client])]
                  ELSE /\ TRUE
                       /\ UNCHANGED << counts, in_channels, out_channels >>

Client(self) == \/ /\ Len(in_channels[self]) < ChannelSize
                   /\ \/ /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], "Place")]
                         /\ counts' = [counts EXCEPT ![self + 1] = counts[self + 1] + 1]
                      \/ /\ IF counts[self + 1] > 0
                               THEN /\ in_channels' = [in_channels EXCEPT ![self] = Append(in_channels[self], "Remove")]
                                    /\ counts' = [counts EXCEPT ![self + 1] = counts[self + 1] - 1]
                               ELSE /\ TRUE
                                    /\ UNCHANGED << counts, in_channels >>
                   /\ UNCHANGED out_channels
                \/ /\ Len(out_channels[self]) > 0
                   /\ IF Head(out_channels[self]) = "Place"
                         THEN /\ counts' = [counts EXCEPT ![self + 1] = counts[self + 1] + 1]
                         ELSE /\ IF Head(out_channels[self]) = "Remove"
                                    THEN /\ counts' = [counts EXCEPT ![self + 1] = counts[self + 1] - 1]
                                    ELSE /\ TRUE
                                         /\ UNCHANGED counts
                   /\ out_channels' = [out_channels EXCEPT ![self] = Tail(out_channels[self])]
                   /\ UNCHANGED in_channels

Next == Server
           \/ (\E self \in Clients: Client(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 

====
