use std::{
    alloc::{GlobalAlloc, Layout, System},
    env::VarError,
    num::{NonZero, NonZeroUsize},
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use roole::ExitValue;

use crate::resources;

const ENV_VAR_NAME: &str = "ROOLE_HEAP_LIMIT";

pub fn init() {
    let heap_limit = compute_heap_limit();
    if let HeapLimitValue::Limited(heap_limit) = heap_limit {
        eprintln!(
            "Heap size limited to {:?} bytes ({})",
            heap_limit, ENV_VAR_NAME
        );
    }
    let _ = HEAP_LIMIT.limit.set(heap_limit);
}

pub fn finish() {
    // do nothing
}

pub fn print_used() {
    if !HEAP_LIMIT.is_definitely_limited() {
        // do not print
        return;
    }
    let max_allocated = HEAP_LIMIT.max_allocated.load(Ordering::SeqCst);

    if max_allocated < 1000 {
        // print bytes
        println!("Maximal allocation sum: {} B", max_allocated);
        return;
    }

    // print kilobytes or above
    let mut whole_part = max_allocated;
    let mut fract_part;

    for prefix in ['k', 'M', 'G', 'T'] {
        (fract_part, whole_part) = (whole_part % 1000, whole_part / 1000);

        if whole_part < 1000 || prefix == 'T' {
            // print with this prefix
            eprintln!(
                "Maximal allocation sum: {}.{:0>3} {}B",
                whole_part, fract_part, prefix
            );
            return;
        }
    }
}

#[derive(Clone, Copy)]
enum HeapLimitValue {
    Limited(NonZeroUsize),
    Unlimited,
}

fn compute_heap_limit() -> HeapLimitValue {
    let mut value = match std::env::var(ENV_VAR_NAME) {
        Ok(value) => value,
        Err(VarError::NotUnicode(_)) => {
            panic!("{} must be Unicode", ENV_VAR_NAME)
        }
        Err(VarError::NotPresent) => {
            // no heap limit
            return HeapLimitValue::Unlimited;
        }
    };

    let Some(unit_prefix) = value.pop() else {
        // consider empty variable unset, no heap limit
        return HeapLimitValue::Unlimited;
    };

    const THOUSAND: u128 = 1000;
    const MILLION: u128 = THOUSAND * THOUSAND;

    let multiplier = match unit_prefix {
        'k' => THOUSAND,
        'M' => MILLION,
        'G' => MILLION * THOUSAND,
        'T' => MILLION * MILLION,
        _ => {
            // return the character back
            value.push(unit_prefix);
            1
        }
    };

    // ensure a bigger heap limit than machine usize can be processed
    let Ok(value) = value.parse::<u128>() else {
        panic!(
            "{} must be an unsigned number with optional postfix k/M/G/T",
            ENV_VAR_NAME
        );
    };

    // compute the number of bytes, saturate
    let num_bytes = value.saturating_mul(multiplier);
    // convert to machine usize, saturate
    let num_bytes = usize::try_from(num_bytes).unwrap_or(usize::MAX);

    let Some(num_bytes) = NonZero::new(num_bytes) else {
        panic!("{} cannot be zero", num_bytes);
    };

    HeapLimitValue::Limited(num_bytes)
}

struct HeapLimit {
    limit: OnceLock<HeapLimitValue>,
    allocated: AtomicUsize,
    max_allocated: AtomicUsize,
    exceeded: AtomicBool,
}

#[global_allocator]
static HEAP_LIMIT: HeapLimit = HeapLimit {
    limit: OnceLock::new(),
    allocated: AtomicUsize::new(0),
    max_allocated: AtomicUsize::new(0),
    exceeded: AtomicBool::new(false),
};

impl HeapLimit {
    fn is_definitely_limited(&self) -> bool {
        self.limit
            .get()
            .is_some_and(|limit| matches!(limit, HeapLimitValue::Limited(_)))
    }

    fn is_definitely_unlimited(&self) -> bool {
        self.limit
            .get()
            .is_some_and(|limit| matches!(limit, HeapLimitValue::Unlimited))
    }
}

unsafe impl GlobalAlloc for HeapLimit {
    /// Allocates memory.
    ///
    /// # Safety
    ///
    /// This function does not unwind: either returns the allocated pointer,
    /// returns a null pointer (leading to memory exhaustion), or prints and exits.
    ///
    /// No calculations are done, everything is forwarded to System allocator.
    ///
    /// There is no reliance on the allocations actually happening.
    ///
    /// # Re-entrance
    ///
    /// This function does not allocate unless the allocation limit is exceeded,
    /// in which case it sets `exceeded` to true before calling functions from std
    /// so that further allocations allocate normally with the limit exceeded.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // if the limit variable has not been set yet, consider that there is a limit with the maximum value
        let limit = self.limit.get().cloned();

        let limit = match limit {
            Some(HeapLimitValue::Unlimited) => {
                // if we know there is no limit, allocate normally without tracking
                // SAFETY: since we are inside `GlobalAlloc::alloc`, we fulfil restrictions for `System.alloc`
                return unsafe { System.alloc(layout) };
            }
            Some(HeapLimitValue::Limited(limit)) => limit,
            None => {
                // no limit specified, assume it is maximum but track allocations
                NonZeroUsize::MAX
            }
        };

        // add layout size to allocated, this can wrap around
        let allocation_size = layout.size();
        let before_allocation = self.allocated.fetch_add(allocation_size, Ordering::SeqCst);

        let Some(after_allocation) = before_allocation.checked_add(allocation_size) else {
            // allocated must have wrapped around, signal memory exhaustion
            return std::ptr::null_mut();
        };

        if after_allocation > limit.get() {
            // limit was breached
            // to be able to print and exit with potential allocations, only fail when not exceeded already
            let exceeded_previously = self.exceeded.fetch_or(true, Ordering::SeqCst);

            if !exceeded_previously {
                // limit breached right now, print used resources, error message, and terminate
                resources::print_used();
                eprintln!("Heap limit exceeded (set by {})", ENV_VAR_NAME);
                std::process::exit(ExitValue::HeapLimitExceeded as i32);
            }
        }

        // allocate normally
        // SAFETY: since we are inside `GlobalAlloc::alloc`, we fulfil restrictions for `System.alloc`
        let allocated_ptr = unsafe { System.alloc(layout) };

        // if allocation failed, subtract the allocation size from allocated
        if allocated_ptr.is_null() {
            self.allocated.fetch_sub(allocation_size, Ordering::SeqCst);
        } else {
            // if allocation did not fail, update max_allocated to be at least after_allocation
            self.max_allocated
                .fetch_max(after_allocation, Ordering::SeqCst);
        }

        // return the allocated pointer
        allocated_ptr
    }

    /// Deallocates memory.
    ///
    /// # Safety
    ///
    /// This function does not unwind, just returns normally.
    ///
    /// No calculations are done, everything is forwarded to System allocator.
    ///
    /// There is no reliance on the allocations actually happening.
    ///
    /// # Re-entrance
    ///
    /// This function does not allocate.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // deallocate normally, this must succeed
        // SAFETY: since we are inside `GlobalAlloc::dealloc`, we fulfil restrictions for `System.dealloc`
        unsafe {
            std::alloc::System.dealloc(ptr, layout);
        }

        // do not track the deallocation if the heap is definitely unlimited
        if self.is_definitely_unlimited() {
            return;
        }

        // subtract the allocation size from allocated
        self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}
