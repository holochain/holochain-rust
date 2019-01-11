pub use std::os::windows::io::RawSocket as RawFd;

pub mod errno {
    use libc::c_int;

    // Use constants as defined in the windows header errno.h
    // libzmq should be compiled with VS2010 SDK headers or newer
    pub const EACCES: c_int = 13;
    pub const EADDRINUSE: c_int = 100;
    pub const EADDRNOTAVAIL: c_int = 101;
    pub const EAGAIN: c_int = 11;
    pub const EBUSY: c_int = 16;
    pub const ECONNREFUSED: c_int = 107;
    pub const EFAULT: c_int = 14;
    pub const EINTR: c_int = 4;
    pub const EHOSTUNREACH: c_int = 110;
    pub const EINPROGRESS: c_int = 112;
    pub const EINVAL: c_int = 22;
    pub const EMFILE: c_int = 24;
    pub const EMSGSIZE: c_int = 115;
    pub const ENAMETOOLONG: c_int = 38;
    pub const ENETDOWN: c_int = 116;
    pub const ENOBUFS: c_int = 119;
    pub const ENODEV: c_int = 19;
    pub const ENOENT: c_int = 2;
    pub const ENOMEM: c_int = 12;
    pub const ENOTCONN: c_int = 126;
    pub const ENOTSOCK: c_int = 128;
    pub const ENOTSUP: c_int = 129;
    pub const EPROTO: c_int = 134;
    pub const EPROTONOSUPPORT: c_int = 135;
}
