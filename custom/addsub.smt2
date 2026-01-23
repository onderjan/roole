(set-logic QF_BV) 

(declare-const x (_ BitVec 6))  
(declare-const y (_ BitVec 6))
(declare-const z (_ BitVec 6))   

(assert (= z (bvsub (bvadd x y) y)))

(assert (not (= x z)))
(check-sat)
; unsat
(exit)