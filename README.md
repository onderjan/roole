# Roole

A solver of Satisfiability Modulo Theories (SMT) with bitvectors in Rust.
Can produce proofs that can be proof-checked by the trusted proof-checker
[Roolean](https://github.com/onderjan/roolean).

Roole and Roolean are currently in a proof-of-concept stage.

## Basic SMT solving

Roole takes [SMT-LIB2](https://smt-lib.org/) problems in the theory
of quantifier-free bitvectors QF_BV. 

Having [Rust](https://rust-lang.org/) installed, install Roole through it:
```console
$ cargo install roole
```

Then, feed the problem to Roole:

```console
$ roole example.smt2
Evaluating file "example.smt2"
Solving SAT problem
Info: 511 nodes, 511 opened (100.000%); 255 inconclusive, 0 pre-learned, 0 pre-resolved, 256 learned; 256 leaves, 256 closed (100.000%); 0 backtrackings
Result: Unsatisfiable
Validating
Validated
unsat
Finished evaluation
----------
----------
```

As per SMT-LIB2 conventions, `unsat` will be printed to standard output, the rest to standard error.

## Trusted results

Use the `--proof-output` flag of Roole to produce a proof certificate of the result:

```console
$ roole --proof-output example.proof example.smt2
Evaluating file "example.smt2"
Solving SAT problem
(...)
Writing proof to "example.proof"
Proof written
unsat
Finished evaluation
----------
----------
```

After installing [Lean](https://lean-lang.org/), clone [Roolean](https://github.com/onderjan/roolean).
In its project directory, execute it with the paths to the problem and the proof certificate:

```console
$ lake exe roolean example.smt2 example.proof
Checking satisfiability
unsat
Execution successful
Roolean finished in 60 ms
```

As previously, `unsat` will be printed to standard output, the rest to standard error.
The result obtained by Roolean is the same, but more trustworthy: 
the reasoning about abstract domains that allows the result to be computed quicker 
than by brute force is proven sound in Roolean
(using the capabilities of Lean as an interactive theorem prover), 
shrinking the surface for bugs.

Roolean uses the proof certificate as an oracle that guides its own solving process,
which alleviates the slowness of solving in Lean compared to Rust.
If the certificate is wrong or tampered with so it no longer proves the problem,
Roolean refuses it:

```console
$ lake exe roolean example.smt2 example_tampered.proof
Checking satisfiability
Roolean error in 65 ms
Execution error: EExecutor.Interpretation (EInterpretation.WrongProofCheck (ECheckResult.Unknown))
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.

(Note that the built binary currently depends on [aws_smt_ir](https://crates.io/crates/aws-smt-ir) 
and other libraries that are Apache-2.0 only.)
