//! Process management syscalls

use crate::{mm::{MapPermission, PTEFlags}, 
    task::{
        attribute_of_vpn, change_program_brk, current_syscall_count, exit_current_and_run_next, insert_framed_area, remove_framed_area, suspend_current_and_run_next}};

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

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    #[inline]
    fn check(id:usize,pteflag:PTEFlags)->bool {
        let va = crate::mm::VirtAddr::from(id);
        let vpn = va.floor();
        let Some(flag) = attribute_of_vpn(vpn) else {
            return false;
        };
        if !flag.contains(PTEFlags::V|PTEFlags::U|pteflag) {
            return false;
        }
        true
    }
    trace!("kernel: sys_trace");
    match trace_request {
    0 => {
        if !check(id,PTEFlags::R) {
            return -1;
        }
        let src = unsafe { &* (id as *const u8) };
        let mut dst= 0u8;
        read_value(src,&mut dst);
        dst as isize
    }
    1 => {
        if !check(id,PTEFlags::W) {
            return -1;
        }
        let v = (data & 0x000000ff) as u8;
        let dst = id as *mut u8;
        let dst = unsafe { &mut *dst };
        write_value(&v,dst);
        // 在这加了一句话，就会卡在这里，不知为，打印 "src:0 dst:"，dst冒号后为空，然后就停下来了
        // 后来发现，这是因为dst其字面值为非法地址
        // println!("src:{} dst:{}",v,dst);
        0}
    2 => {
        current_syscall_count(id)
    }
    _ => -1
    }
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
    let e = crate::mm::VirtAddr::from(start+len);
    let start_vpn = s.floor();
    let end_vpn = e.ceil();
    let vpn_range = crate::mm::VPNRange::new(start_vpn, end_vpn);
    for vpn in vpn_range {
        let Some(flag) = attribute_of_vpn(vpn) else {
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
    insert_framed_area(s,e,perm);

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
    if remove_framed_area(s, e) {
        0
    } else {
        -1
    }
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
