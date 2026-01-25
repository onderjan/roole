(set-logic QF_BV) 

(declare-const x (_ BitVec 9))  
(declare-const y (_ BitVec 9))
(declare-const z (_ BitVec 9))   

(assert (= y (bvmul x #b000000010)))
(assert (= z (bvadd x x)))

(assert (not (= y z)))
(check-sat)
; unsat
(exit)