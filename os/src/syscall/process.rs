//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str, MapPermission, PTEFlags},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskControlBlock,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    let Some((data,_name)) = get_app_data_by_name(path.as_str()) else {
        return -1;
    };
    let task = current_task().unwrap();
    task.exec(data/*,name */);
    0
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}
fn write_value<T>(src:&T,dst:&mut T) {
    let src = src as *const T as *const u8;
    let dst = dst as * const T as *mut u8;
    let len = core::mem::size_of::<T>();
    let src = unsafe { core::slice::from_raw_parts(src, len) };
    let dst = unsafe { core::slice::from_raw_parts_mut(dst, len) };
    write_bytes(src,dst)
}
fn write_bytes(src:&[u8],dst:&mut [u8]) {
    use crate::mm::translated_byte_buffer;
    use crate::task::current_user_token;

    let len = core::cmp::min(dst.len(),src.len());
    let dst = dst.as_mut_ptr();
    let bytes_dst = translated_byte_buffer(current_user_token(), dst, len);
    let bytes_src = src;

    let mut s = 0;
    for bytes in bytes_dst {
        let len = bytes.len();
        bytes.copy_from_slice(&bytes_src[s..s+len]);
        s += len;
    }
}

#[allow(unused)]
fn read_value<T>(src:&T, dst:&mut T) {
    let src = src as *const T as *const u8;
    let dst = dst as *const T as *mut u8;
    let len = core::mem::size_of::<T>();
    let src = unsafe { core::slice::from_raw_parts(src, len) };
    let dst = unsafe { core::slice::from_raw_parts_mut(dst, len) };
    read_bytes(src,dst)
}

fn read_bytes(src:&[u8],dst:&mut[u8]) {
    use crate::mm::translated_byte_buffer;
    use crate::task::current_user_token;

    let len = core::cmp::min(src.len(),dst.len());
    let src = src.as_ptr();
    let bytes_src = translated_byte_buffer(current_user_token(), src, len);
    let bytes_dst = dst;

    let mut s = 0;
    for bytes in bytes_src {
        let len = bytes.len();
        bytes_dst[s..s+len].copy_from_slice(bytes);
        s += len;
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = crate::timer::get_time_us();
    let timeval = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let dst = unsafe { ts.as_mut().unwrap() };
    write_value(&timeval,dst);

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap IMPLEMENTED here!");
    let s = crate::mm::VirtAddr::from(start);
    if !s.aligned() {
        return -1;
    }
    if prot & !0x7 != 0 {
        return -1;
    }
    if prot & 0x7 == 0 {
        return -1;
    }
    let Some(task) = current_task() else {
        return -1;
    };
    let ms = &mut task.inner_exclusive_access().memory_set;
    let e = crate::mm::VirtAddr::from(start+len);
    let start_vpn = s.floor();
    let end_vpn = e.ceil();
    let vpn_range = crate::mm::VPNRange::new(start_vpn, end_vpn);
    for vpn in vpn_range {
        let Some(flag) = ms.pteflag(vpn) else {
            continue;
        };
        if flag.contains(PTEFlags::V) {
            return -1;
        }
    }


    let prot = prot << 1;
    let Some(perm) = MapPermission::from_bits(prot as u8) else {
        return -1;
    };
    let perm = perm|MapPermission::U;
    ms.insert_framed_area(s,e,perm);

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap IMPLEMENTED here!");

    let s = crate::mm::VirtAddr::from(start);
    if !s.aligned() {
        return -1;
    }
    let e = crate::mm::VirtAddr::from(start+len);

    let Some(task) = current_task() else {
        return -1;
    };
    let ms = &mut task.inner_exclusive_access().memory_set;

    if ms.remove_framed_area(s, e, MapPermission::U) {
        0
    } else {
        -1
    }
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, path);
    let Some((data,_name)) = get_app_data_by_name(path.as_str()) else {
        return -1;
    };
    let task = current_task().unwrap();
    let new_task = TaskControlBlock::spawn(Some(task), data/*, name */);
    let pid = new_task.getpid() as isize;
    add_task(new_task);
    pid
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if prio > 1 {
        current_task()
        .unwrap()
        .inner_exclusive_access()
        .priority = prio as usize;
        prio
    } else {
        -1
    }
}
