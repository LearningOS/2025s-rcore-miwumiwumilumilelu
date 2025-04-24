//! Process management syscalls
use crate::{ mm::{user_ptr_to_kernel_ref, translated_byte_buffer}
            ,task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,current_user_token,
                    select_cur_task_to_mmap, select_cur_task_to_munmap}
            ,timer::get_time_us
            ,config::MAX_SYSCALL_NUM,
            };

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let token = current_user_token();
    let ts = user_ptr_to_kernel_ref(token, _ts );
    *ts = TimeVal {
        sec : us / 1_000_000,
        usec : us % 1_000_000,
    };
    0    
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    match _trace_request {
        0 => {
            // id 应被视作 *const u8，读取地址处的值
            let buffers = translated_byte_buffer(token, _id as *const u8, 1);
            if let Some(buffer) = buffers.first() {
                buffer[0] as isize
            } else {
                -1 // 地址不可见或不可读
            }
        }
        1 => {
            // id 应被视作 *mut u8，写入 data 的最低字节
            let mut buffers = translated_byte_buffer(token, _id as *const u8, 1);
            if let Some(buffer) = buffers.first_mut() {
                buffer[0] = (_data & 0xff) as u8;
                0 // 成功
            } else {
                -1 // 地址不可见或不可写
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

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if _len == 0 {
        return 0;
    }
    //port只能是0x1,0x3,0x5,0x7
    // 0x1: read
    // 0x3: read and write
    // 0x5: read and execute
    // 0x7: read, write and execute
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    }
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    select_cur_task_to_mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    select_cur_task_to_munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
