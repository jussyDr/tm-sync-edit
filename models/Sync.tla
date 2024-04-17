---- MODULE Sync ---- 
EXTENDS Integers, Sequences
CONSTANTS NumClients, MaxQueueLen, NumBlockTypes
RECURSIVE Broadcast(_, _)

Broadcast(clientQueues, block) == 
  IF clientQueues = <<>> THEN
    clientQueues
  ELSE
    <<Append(Head(clientQueues), block)>> \o Broadcast(Tail(clientQueues), block)

(* --algorithm Sync

variables 
  serverMap = 0;
  serverQueue = <<>>;
  clientMaps = [client \in 1..NumClients |-> 0];
  clientQueues = [client \in 1..NumClients |-> <<>>];

define 
  QueueLenConstraint ==
    /\ Len(serverQueue) <= MaxQueueLen
    /\ \A client \in 1..NumClients:
      Len(clientQueues[client]) <= MaxQueueLen
  
  StateInvariant ==
    IF 
      /\ serverQueue = <<>> 
      /\ \A client \in 1..NumClients: 
        clientQueues[client] = <<>> 
    THEN
      \A client \in 1..NumClients:
        clientMaps[client] = serverMap
    ELSE
      TRUE
end define;

process Server = 0 
begin
  Run:
    await serverQueue # <<>>;
    serverMap := Head(serverQueue);
    clientQueues := Broadcast(clientQueues, Head(serverQueue));
    serverQueue := Tail(serverQueue);
    goto Run;
end process;

process Client \in (1..NumClients) 
begin
  Run:
    either
      if clientQueues[self] # <<>> then
        clientMaps[self] := Head(clientQueues[self]);
        clientQueues[self] := Tail(clientQueues[self]);
      end if;
    or
      with block \in 0..NumBlockTypes - 1 do
        serverQueue := Append(serverQueue, block);
      end with;
    end either;
    goto Run;
end process;

end algorithm *)
\* BEGIN TRANSLATION (chksum(pcal) = "83d23bd2" /\ chksum(tla) = "7f43de36")
\* Label Run of process Server at line 41 col 5 changed to Run_
VARIABLES serverMap, serverQueue, clientMaps, clientQueues, pc

(* define statement *)
QueueLenConstraint ==
  /\ Len(serverQueue) <= MaxQueueLen
  /\ \A client \in 1..NumClients:
    Len(clientQueues[client]) <= MaxQueueLen

StateInvariant ==
  IF
    /\ serverQueue = <<>>
    /\ \A client \in 1..NumClients:
      clientQueues[client] = <<>>
  THEN
    \A client \in 1..NumClients:
      clientMaps[client] = serverMap
  ELSE
    TRUE


vars == << serverMap, serverQueue, clientMaps, clientQueues, pc >>

ProcSet == {0} \cup ((1..NumClients))

Init == (* Global variables *)
        /\ serverMap = 0
        /\ serverQueue = <<>>
        /\ clientMaps = [client \in 1..NumClients |-> 0]
        /\ clientQueues = [client \in 1..NumClients |-> <<>>]
        /\ pc = [self \in ProcSet |-> CASE self = 0 -> "Run_"
                                        [] self \in (1..NumClients) -> "Run"]

Run_ == /\ pc[0] = "Run_"
        /\ serverQueue # <<>>
        /\ serverMap' = Head(serverQueue)
        /\ clientQueues' = Broadcast(clientQueues, Head(serverQueue))
        /\ serverQueue' = Tail(serverQueue)
        /\ pc' = [pc EXCEPT ![0] = "Run_"]
        /\ UNCHANGED clientMaps

Server == Run_

Run(self) == /\ pc[self] = "Run"
             /\ \/ /\ IF clientQueues[self] # <<>>
                         THEN /\ clientMaps' = [clientMaps EXCEPT ![self] = Head(clientQueues[self])]
                              /\ clientQueues' = [clientQueues EXCEPT ![self] = Tail(clientQueues[self])]
                         ELSE /\ TRUE
                              /\ UNCHANGED << clientMaps, clientQueues >>
                   /\ UNCHANGED serverQueue
                \/ /\ \E block \in 0..NumBlockTypes - 1:
                        serverQueue' = Append(serverQueue, block)
                   /\ UNCHANGED <<clientMaps, clientQueues>>
             /\ pc' = [pc EXCEPT ![self] = "Run"]
             /\ UNCHANGED serverMap

Client(self) == Run(self)

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == Server
           \/ (\E self \in (1..NumClients): Client(self))
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 

====
