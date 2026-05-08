use roole::ExitValue;

pub fn exit_value_str(exit_value: ExitValue) -> &'static str {
    match exit_value {
        ExitValue::Success => "success",
        ExitValue::Satisfiable => "sat",
        ExitValue::Unsatisfiable => "unsat",
        ExitValue::Unknown => "unknown",
        ExitValue::TimeLimitExceeded => "time_limit",
        ExitValue::HeapLimitExceeded => "heap_limit",
        ExitValue::Panic => "panic",
    }
}
