const std = @import("std");
const c = @cImport({
    @cInclude("sys/resource.h");
    @cInclude("sys/stat.h");
    @cInclude("errno.h");
    @cInclude("fcntl.h");
    @cInclude("limits.h");
    @cInclude("stdio.h");
    @cInclude("stdlib.h");
    @cInclude("string.h");
    @cInclude("unistd.h");
});

fn countOpenFiles(num: c_int) !i32 {
    var stats: c.struct_stat = undefined;
    var count: i32 = 0;
    var i: c_int = 0;
    while (i < num) : (i += 1) {
        if (c.fstat(i, &stats) == 0) {
            std.debug.print("Currently open: fd #{d} (inode {d})\n", .{ i, stats.st_ino });
            count += 1;
        }
    }
    return count;
}

fn openFiles(num: c_int) !void {
    const count = try countOpenFiles(num);
    var fd: c_int = undefined;

    std.debug.print("Currently open files: {d}\n", .{count});

    var i = count;
    while (i <= num) : (i += 1) {
        fd = c.open("/dev/null", c.O_RDONLY);
        if (fd < 0) {
            const errno_value = @intFromEnum(std.posix.errno(fd));
            if (errno_value == c.EMFILE) {
                std.debug.print(
                    "Opened {d} additional files, then failed: {s} ({d})\n",
                    .{ i - count, c.strerror(errno_value), errno_value },
                );
                break;
            } else {
                std.debug.print(
                    "Unable to open '/dev/null' on fd#{d}: {s} (errno {d})\n",
                    .{ i, c.strerror(errno_value), errno_value },
                );
                break;
            }
        }
    }
}

pub fn main() !void {
    var openmax: c_int = undefined;
    var rlp: c.struct_rlimit = undefined;
    var stdout_ = std.io.getStdOut();

    if (@hasDecl(c, "OPEN_MAX")) {
        std.debug.print("OPEN_MAX is defined as {d}.\n", .{c.OPEN_MAX});
    } else {
        std.debug.print("OPEN_MAX is not defined on this platform.\n", .{});
    }

    try stdout_.writeAll("'getconf OPEN_MAX' says: ");
    _ = c.system("getconf OPEN_MAX");

    const sysconf_result = c.sysconf(c._SC_OPEN_MAX);
    if (sysconf_result < 0) {
        const errno_value = @intFromEnum(std.posix.errno(sysconf_result));
        if (errno_value == 0) {
            std.log.err("sysconf(3) considers _SC_OPEN_MAX unsupported?\n", .{});
        } else {
            std.log.err("sysconf(3) error for _SC_OPEN_MAX: {s}\n", .{c.strerror(errno_value)});
        }
        std.process.exit(c.EXIT_FAILURE);
    } else {
        openmax = @intCast(sysconf_result);
    }

    std.debug.print("sysconf(3) says this process can open {d} files.\n", .{openmax});

    const fd = c.getrlimit(c.RLIMIT_NOFILE, &rlp);
    if (fd != 0) {
        const errno_value = @intFromEnum(std.posix.errno(fd));
        std.log.err("Unable to get per process rlimit: {s}\n", .{c.strerror(errno_value)});
        std.process.exit(c.EXIT_FAILURE);
    }
    openmax = @intCast(rlp.rlim_cur);
    std.debug.print(
        "getrlimit(2) says this process can open {d} files.\n",
        .{openmax},
    );

    std.debug.print("Which one is it?\n\n", .{});

    try openFiles(openmax);
    std.process.exit(c.EXIT_SUCCESS);
}
