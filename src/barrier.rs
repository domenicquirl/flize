#[cfg(target_os = "linux")]
pub use linux::{light_barrier, strong_barrier};

#[cfg(target_os = "linux")]
mod linux {
    use lazy_static::lazy_static;
    use std::sync::atomic::{compiler_fence, fence, Ordering};

    #[inline]
    pub fn strong_barrier() {
        match *STRATEGY {
            Strategy::Membarrier => membarrier::barrier(),
            Strategy::Fallback => fence(Ordering::SeqCst),
        }
    }

    #[inline]
    pub fn light_barrier() {
        match *STRATEGY {
            Strategy::Membarrier => compiler_fence(Ordering::SeqCst),
            Strategy::Fallback => fence(Ordering::SeqCst),
        }
    }

    enum Strategy {
        Membarrier,
        Fallback,
    }

    lazy_static! {
        static ref STRATEGY: Strategy = if membarrier::is_supported() {
            Strategy::Membarrier
        } else {
            Strategy::Fallback
        };
    }

    mod membarrier {
        #[repr(i32)]
        #[allow(dead_code, non_camel_case_types)]
        enum membarrier_cmd {
            MEMBARRIER_CMD_QUERY = 0,
            MEMBARRIER_CMD_GLOBAL = 1,
            MEMBARRIER_CMD_GLOBAL_EXPEDITED = 1 << 1,
            MEMBARRIER_CMD_REGISTER_GLOBAL_EXPEDITED = 1 << 2,
            MEMBARRIER_CMD_PRIVATE_EXPEDITED = 1 << 3,
            MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED = 1 << 4,
            MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE = 1 << 5,
            MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE = 1 << 6,
        }

        fn sys_membarrier(cmd: membarrier_cmd) -> libc::c_long {
            unsafe { libc::syscall(libc::SYS_membarrier, cmd as libc::c_int, 0 as libc::c_int) }
        }

        pub fn is_supported() -> bool {
            // Queries which membarrier commands are supported. Checks if private expedited
            // membarrier is supported.
            let ret = sys_membarrier(membarrier_cmd::MEMBARRIER_CMD_QUERY);
            if ret < 0
                || ret & membarrier_cmd::MEMBARRIER_CMD_PRIVATE_EXPEDITED as libc::c_long == 0
                || ret & membarrier_cmd::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED as libc::c_long
                    == 0
            {
                return false;
            }

            // Registers the current process as a user of private expedited membarrier.
            if sys_membarrier(membarrier_cmd::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED) < 0 {
                return false;
            }

            true
        }

        macro_rules! fatal_assert {
            ($cond:expr) => {
                if !$cond {
                    #[allow(unused_unsafe)]
                    unsafe {
                        libc::abort();
                    }
                }
            };
        }

        pub fn barrier() {
            fatal_assert!(sys_membarrier(membarrier_cmd::MEMBARRIER_CMD_PRIVATE_EXPEDITED) >= 0);
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod fallback {
    use std::sync::atomic::{fence, Ordering};

    #[inline]
    pub fn strong_barrier() {
        fence(Ordering::SeqCst);
    }

    #[inline]
    pub fn light_barrier() {
        fence(Ordering::SeqCst);
    }
}
