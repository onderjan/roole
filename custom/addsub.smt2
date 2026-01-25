(set-logic QF_BV) 

(declare-const x (_ BitVec 9))  
(declare-const y (_ BitVec 9))
(declare-const z (_ BitVec 9))   

(assert (= z (bvsub (bvadd x y) y)))

(assert (not (= x z)))
(check-sat)
; unsat
(exit)