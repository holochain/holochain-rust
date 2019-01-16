pub mod base;
pub mod admin;

pub use self::base::{mount_container_from_config, CONTAINER, Container};
pub use self::admin::ContainerAdmin;

#[cfg(test)]
pub use self::base::tests;