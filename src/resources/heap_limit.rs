use std::{
    alloc::{GlobalAlloc, Layout, System},
    env::VarError,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use roole::ExitValue;

const ENV_VAR_NAME: &str = "ROOLE_HEAP_LIMIT";

pub fn init_heap_limit() {
    let heap_limit = compute_heap_limit();
    if heap_limit != usize::MAX {
        eprintln!(
            "Heap size limited to {:?} bytes ({})",
            heap_limit, ENV_VAR_NAME
        );
    }
    let _ = HEAP_LIMIT.limit.set(heap_limit);
}

fn compute_heap_limit() -> usize {
    let mut value = match std::env::var(ENV_VAR_NAME) {
        Ok(value) => value,
        Err(VarError::NotUnicode(_)) => {
            panic!("{} must be Unicode", ENV_VAR_NAME)
        }
        Err(VarError::NotPresent) => {
            // no heap limit
            return usize::MAX;
        }
    };

    let Some(unit_prefix) = value.pop() else {
        // consider empty variable unset, no heap limit
        return usize::MAX;
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

    let Some(num_bytes) = value.checked_mul(multiplier) else {
        // use the maximum value
        return usize::MAX;
    };
    // convert to machine usize, use maximum value if it is bigger
    usize::try_from(num_bytes).unwrap_or(usize::MAX)
}

struct HeapLimit {
    limit: OnceLock<usize>,
    allocated: AtomicUsize,
    exceeded: AtomicBool,
}

#[global_allocator]
static HEAP_LIMIT: HeapLimit = HeapLimit {
    limit: OnceLock::new(),
    allocated: AtomicUsize::new(0),
    exceeded: AtomicBool::new(false),
};

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
        // if the limit has not been set yet, consider that there is a limit with the maximum value
        let limit = self.limit.get().cloned();

        // if the limit has been already set to maximum, allocate normally without tracking
        if limit.is_some_and(|limit| limit == usize::MAX) {
            // SAFETY: since we are inside `GlobalAlloc::alloc`, we fulfil restrictions for `System.alloc`
            return unsafe { System.alloc(layout) };
        };

        // if the limit has not been set yet, assume it is maximum but track allocations
        let limit = limit.unwrap_or(usize::MAX);

        let allocation_size = layout.size();

        // add layout size to allocated, this can wrap around
        let before_allocation = self.allocated.fetch_add(allocation_size, Ordering::SeqCst);

        let Some(after_allocation) = before_allocation.checked_add(allocation_size) else {
            // allocated must have wrapped around, signal memory exhaustion
            return std::ptr::null_mut();
        };

        if after_allocation > limit {
            // limit was breached
            // to be able to print and exit with potential allocations, only fail when not exceeded already
            let exceeded_previously = self.exceeded.fetch_or(true, Ordering::SeqCst);

            if !exceeded_previously {
                // limit breached right now, print a message and terminate
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

        // do not track the deallocation if the allowed heap size has been definitely set to maximum
        if let Some(limit) = self.limit.get()
            && *limit == usize::MAX
        {
            return;
        }

        // subtract the allocation size from allocated
        self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}
