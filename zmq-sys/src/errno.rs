#[cfg(unix)]
use libc as errno;
#[cfg(windows)]
use windows::errno;

const ZMQ_HAUSNUMERO: i32 = 156_384_712;

pub const EACCES:           i32 = errno::EACCES;
pub const EADDRINUSE:       i32 = errno::EADDRINUSE;
pub const EAGAIN:           i32 = errno::EAGAIN;
pub const EBUSY:            i32 = errno::EBUSY;
pub const ECONNREFUSED:     i32 = errno::ECONNREFUSED;
pub const EFAULT:           i32 = errno::EFAULT;
pub const EINTR:            i32 = errno::EINTR;
pub const EHOSTUNREACH:     i32 = errno::EHOSTUNREACH;
pub const EINPROGRESS:      i32 = errno::EINPROGRESS;
pub const EINVAL:           i32 = errno::EINVAL;
pub const EMFILE:           i32 = errno::EMFILE;
pub const EMSGSIZE:         i32 = errno::EMSGSIZE;
pub const ENAMETOOLONG:     i32 = errno::ENAMETOOLONG;
pub const ENODEV:           i32 = errno::ENODEV;
pub const ENOENT:           i32 = errno::ENOENT;
pub const ENOMEM:           i32 = errno::ENOMEM;
pub const ENOTCONN:         i32 = errno::ENOTCONN;
pub const ENOTSOCK:         i32 = errno::ENOTSOCK;
#[cfg(not(target_os = "openbsd"))]
pub const EPROTO:           i32 = errno::EPROTO;
#[cfg(target_os = "openbsd")]
pub const EPROTO:           i32 = errno::EOPNOTSUPP;
pub const EPROTONOSUPPORT:  i32 = errno::EPROTONOSUPPORT;

#[cfg(not(target_os = "windows"))]
pub const ENOTSUP:          i32 = (ZMQ_HAUSNUMERO + 1);
#[cfg(target_os = "windows")]
pub const ENOTSUP:          i32 = errno::ENOTSUP;

pub const ENOBUFS:          i32 = errno::ENOBUFS;
pub const ENETDOWN:         i32 = errno::ENETDOWN;
pub const EADDRNOTAVAIL:    i32 = errno::EADDRNOTAVAIL;

// native zmq error codes
pub const EFSM:             i32 = (ZMQ_HAUSNUMERO + 51);
pub const ENOCOMPATPROTO:   i32 = (ZMQ_HAUSNUMERO + 52);
pub const ETERM:            i32 = (ZMQ_HAUSNUMERO + 53);
pub const EMTHREAD:         i32 = (ZMQ_HAUSNUMERO + 54);
