use libc::RLIM_INFINITY;
use nix::sys::resource::{getrlimit, Resource};
use nix::sys::stat::fstat;
use nix::unistd::{sysconf, SysconfVar};
use std::fs::File;
use std::io::{self, ErrorKind, Write};
use std::process::{self, Command};

#[cfg(target_os = "linux")]
fn get_open_max_sysconf() -> io::Result<usize> {
    match sysconf(SysconfVar::OPEN_MAX) {
        Ok(Some(val)) => Ok(val as usize),
        Ok(None) => Err(io::Error::new(
            ErrorKind::Other,
            "sysconf did not return a value",
        )),
        Err(e) => Err(io::Error::new(
            ErrorKind::Other,
            format!("sysconf failed: {}", e),
        )),
    }
}

#[cfg(target_os = "macos")]
fn get_open_max_sysconf() -> io::Result<usize> {
    // On macOS, we can use sysctl to get the maximum number of open files
    let output = Command::new("sysctl")
        .arg("-n")
        .arg("kern.maxfilesperproc")
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("sysctl failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let max_files = String::from_utf8_lossy(&output.stdout);
    max_files.trim().parse::<usize>().map_err(|e| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to parse sysctl output: {}", e),
        )
    })
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_open_max_sysconf() -> io::Result<usize> {
    Err(io::Error::new(
        ErrorKind::Other,
        "get_open_max_sysconf not implemented for this platform",
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_open_max_rlimit() -> io::Result<usize> {
    match getrlimit(Resource::RLIMIT_NOFILE) {
        Ok((soft_limit, _hard_limit)) => {
            // Check for RLIM_INFINITY
            if soft_limit == RLIM_INFINITY as u64 {
                // calling the fallback function in this case would cause infinite recursion
                // Just return a sensible large value
                // (Linux typically returns a very large number in this case anyway)
                return Ok(usize::MAX);
            } else {
                Ok(soft_limit as usize)
            }
        }
        Err(e) => Err(io::Error::new(
            ErrorKind::Other,
            format!("getrlimit failed: {}", e),
        )),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_open_max_rlimit() -> io::Result<usize> {
    Err(io::Error::new(
        ErrorKind::Other,
        "get_open_max_rlimit not implemented for this platform",
    ))
}

fn count_open_files(num: i32) -> nix::Result<usize> {
    let mut count = 0;
    for i in 0..num {
        let fd = i;
        if let Ok(stats) = fstat(fd) {
            println!("Currently open: fd #{} (inode {})", i, stats.st_ino);
            count += 1;
        }
    }
    Ok(count)
}

fn open_files(num: usize) -> io::Result<()> {
    let count = count_open_files(num as i32)?;

    println!("Currently open files: {}", count);

    let mut open_files = Vec::new(); // Vector to store the open File objects

    for i in count..=num {
        match File::open("/dev/null") {
            Ok(file) => {
                open_files.push(file);
            }
            Err(e) => {
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                let error_code = libc::EMFILE;
                #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                let error_code = 0;

                if e.raw_os_error() == Some(error_code) {
                    println!(
                        "Opened {} additional files, then failed: {} ({:?})",
                        i - count,
                        e,
                        e.kind()
                    );
                    break;
                } else {
                    eprintln!(
                        "Unable to open '/dev/null' on fd#{}: {} (error {:?})",
                        i,
                        e,
                        e.kind()
                    );
                    break;
                }
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Get OPEN_MAX from getconf
    print!("'getconf OPEN_MAX' says: ");
    io::stdout().flush()?;
    let _ = Command::new("getconf").arg("OPEN_MAX").status()?;

    // Get OPEN_MAX using sysconf
    let open_max_sysconf = match get_open_max_sysconf() {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error getting open file limit from sysconf: {}", e);
            process::exit(1);
        }
    };
    println!(
        "sysconf says this process can open {} files.",
        open_max_sysconf
    );

    // Get OPEN_MAX using getrlimit
    let open_max_rlimit = match get_open_max_rlimit() {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error getting open file limit from rlimit: {}", e);
            process::exit(1);
        }
    };
    println!(
        "getrlimit says this process can open {} files.",
        open_max_rlimit
    );

    println!("Which one is it?\n");

    open_files(open_max_rlimit)?;

    Ok(())
}
