---- MODULE blocks ----
EXTENDS Sequences

(* --algorithm blocks

variables
  queue = <<>>;

process server = 0
begin
  Run:
    queue := Append(queue, 1);
end process;

process client \in {1} 
begin
  Run:
    queue := Tail(queue);
end process;

end algorithm *)
\* BEGIN TRANSLATION (chksum(pcal) = "fce56b4c" /\ chksum(tla) = "1c61740a")
\* Label Run of process server at line 12 col 5 changed to Run_
VARIABLES queue, pc

vars == << queue, pc >>

ProcSet == {0} \cup ({1})

Init == (* Global variables *)
        /\ queue = <<>>
        /\ pc = [self \in ProcSet |-> CASE self = 0 -> "Run_"
                                        [] self \in {1} -> "Run"]

Run_ == /\ pc[0] = "Run_"
        /\ queue' = Append(queue, 1)
        /\ pc' = [pc EXCEPT ![0] = "Done"]

server == Run_

Run(self) == /\ pc[self] = "Run"
             /\ queue' = Tail(queue)
             /\ pc' = [pc EXCEPT ![self] = "Done"]

client(self) == Run(self)

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == server
           \/ (\E self \in {1}: client(self))
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 
==== ;
