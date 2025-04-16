//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
    config::MAX_SYSCALL_NUM,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");

    match _trace_request {
        0 => {
            // id 应被视作 *const u8，读取地址处的值
            let id_ptr = _id as *const u8;
            unsafe {
                if let Some(value) = id_ptr.as_ref() {
                    *value as isize
                } else {
                    -1
                }
            }
        }
        1 => {
            // id 应被视作 *mut u8，写入 data 的最低字节
            let id_ptr = _id as *mut u8;
            unsafe {
                if let Some(value) = id_ptr.as_mut() {
                    *value = (_data & 0xff) as u8;
                    0
                } else {
                    -1
                }
            }
        }
        2 => {
            // 查询当前任务调用编号为 id 的系统调用次数
            if _id >= MAX_SYSCALL_NUM {
                return -1;
            }
            let syscall_times = crate::task::get_sys_call_times();
            syscall_times[_id] as isize
        }
        _ => {
            -1
        }
    }
}
