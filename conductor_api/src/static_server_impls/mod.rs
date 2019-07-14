pub mod hyper_staticfile_server;
pub mod nickel_static_server;
pub use self::hyper_staticfile_server::HyperStaticServer;
pub use self::nickel_static_server::NickelStaticServer;
