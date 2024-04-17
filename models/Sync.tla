---- MODULE Sync ---- 
EXTENDS Sequences

(* --algorithm Sync

variables
  queue = <<>>;
  total = 0;
 
process writer = 1
begin
  AddToQueue:
    queue := Append(queue, 1);
end process;

end algorithm *)
\* BEGIN TRANSLATION (chksum(pcal) = "d00eb236" /\ chksum(tla) = "bb6827c")
VARIABLES queue, total, pc

vars == << queue, total, pc >>

ProcSet == {1}

Init == (* Global variables *)
        /\ queue = <<>>
        /\ total = 0
        /\ pc = [self \in ProcSet |-> "AddToQueue"]

AddToQueue == /\ pc[1] = "AddToQueue"
              /\ queue' = Append(queue, 1)
              /\ pc' = [pc EXCEPT ![1] = "Done"]
              /\ total' = total

writer == AddToQueue

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == writer
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 

====
