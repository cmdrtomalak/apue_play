#![allow(non_camel_case_types)]

extern crate libc;

use libc::{
    c_char, c_int, rlimit, stat, system, EMFILE, O_RDONLY, RLIMIT_NOFILE, STDERR_FILENO,
    STDOUT_FILENO,
};
use std::ffi::CStr;
use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::process;

unsafe fn count_open_files(num: c_int) -> c_int {
    let mut stats: MaybeUninit<stat> = MaybeUninit::uninit();
    let mut count: c_int = 0;
    for i in 0..num {
        if libc::fstat(i, stats.as_mut_ptr()) == 0 {
            let stats = stats.assume_init();
            println!("Currently open: fd #{} (inode {})", i, stats.st_ino);
            count += 1;
        }
    }
    count
}

unsafe fn open_files(num: c_int) {
    let count = count_open_files(num);

    println!("Currently open files: {}", count);

    for i in count..=num {
        let fd = libc::open("/dev/null\0".as_ptr() as *const c_char, O_RDONLY);
        if fd < 0 {
            // Get errno using platform-specific method
            let errno_val = if cfg!(target_os = "macos") {
                *libc::__errno_location()
            } else {
                *libc::__errno_location()
            };

            if errno_val == EMFILE {
                println!(
                    "Opened {} additional files, then failed: {} ({})",
                    i - count,
                    CStr::from_ptr(libc::strerror(errno_val)).to_string_lossy(),
                    errno_val
                );
                break;
            } else {
                eprintln!(
                    "Unable to open '/dev/null' on fd#{}: {} (errno {})",
                    i,
                    CStr::from_ptr(libc::strerror(errno_val)).to_string_lossy(),
                    errno_val
                );
                break;
            }
        }
    }
}

fn main() {
    unsafe {
        let open_max: c_int;
        let mut rlp: MaybeUninit<rlimit> = MaybeUninit::uninit();

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            println!("OPEN_MAX is not defined on this platform.\n");
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            println!("OPEN_MAX is not defined on this platform.\n");
        }

        print!("'getconf OPEN_MAX' says: ");
        io::stdout().flush().unwrap();
        system("getconf OPEN_MAX\0".as_ptr() as *const c_char);

        let sysconf_result = libc::sysconf(libc::_SC_OPEN_MAX);
        if sysconf_result < 0 {
            // Get errno using platform-specific method
            let errno_val = if cfg!(target_os = "macos") {
                *libc::__errno_location()
            } else {
                *libc::__errno_location()
            };

            if errno_val == 0 {
                libc::write(
                    STDERR_FILENO,
                    "sysconf(3) considers _SC_OPEN_MAX unsupported?\n\0".as_ptr() as *const _,
                    47,
                );
            } else {
                libc::write(
                    STDERR_FILENO,
                    "sysconf(3) error for _SC_OPEN_MAX: \0".as_ptr() as *const _,
                    35,
                );
                libc::write(
                    STDERR_FILENO,
                    CStr::from_ptr(libc::strerror(errno_val))
                        .to_bytes_with_nul()
                        .as_ptr() as *const _,
                    CStr::from_ptr(libc::strerror(errno_val)).to_bytes().len() + 1,
                );
                libc::write(STDERR_FILENO, "\n\0".as_ptr() as *const _, 2);
            }
            process::exit(libc::EXIT_FAILURE);
        } else {
            open_max = sysconf_result as c_int;
        }

        println!("sysconf(3) says this process can open {} files.", open_max);

        if libc::getrlimit(RLIMIT_NOFILE, rlp.as_mut_ptr()) != 0 {
            // Get errno using platform-specific method
            let errno_val = if cfg!(target_os = "macos") {
                *libc::__errno_location()
            } else {
                *libc::__errno_location()
            };

            libc::write(
                STDERR_FILENO,
                "Unable to get per process rlimit: \0".as_ptr() as *const _,
                35,
            );
            libc::write(
                STDERR_FILENO,
                CStr::from_ptr(libc::strerror(errno_val))
                    .to_bytes_with_nul()
                    .as_ptr() as *const _,
                CStr::from_ptr(libc::strerror(errno_val)).to_bytes().len() + 1,
            );
            libc::write(STDERR_FILENO, "\n\0".as_ptr() as *const _, 2);
            process::exit(libc::EXIT_FAILURE);
        }

        let rlp = rlp.assume_init();
        let open_max = rlp.rlim_cur as c_int;
        println!(
            "getrlimit(2) says this process can open {} files.",
            open_max
        );

        println!("Which one is it?\n");

        open_files(open_max);

        process::exit(libc::EXIT_SUCCESS);
    }
}
