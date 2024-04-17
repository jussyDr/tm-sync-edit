---- MODULE Sync ---- 
EXTENDS Integers, Sequences
CONSTANTS MaxQueueLen

(* --algorithm Sync

variables 
  serverQueue = <<>>;

define QueueLenConstraint ==
  Len(serverQueue) <= MaxQueueLen
end define;

process Server = 0 
begin
  Run:
    serverQueue := Append(serverQueue, 0);
    goto Run;
end process;

end algorithm *)
\* BEGIN TRANSLATION (chksum(pcal) = "f5d7fc90" /\ chksum(tla) = "776a6d4e")
VARIABLES serverQueue, pc

(* define statement *)
     QueueLenConstraint ==
Len(serverQueue) <= MaxQueueLen


vars == << serverQueue, pc >>

ProcSet == {0}

Init == (* Global variables *)
        /\ serverQueue = <<>>
        /\ pc = [self \in ProcSet |-> "Run"]

Run == /\ pc[0] = "Run"
       /\ serverQueue' = Append(serverQueue, 0)
       /\ pc' = [pc EXCEPT ![0] = "Run"]

Server == Run

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == Server
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 

====
