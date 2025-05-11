pub mod builder;
pub mod core;
pub mod preset;

#[doc(inline)]
pub use builder::PluginPolicyBuilder;
#[doc(inline)]
pub use core::{AttrChecker, NodeChecker, PluginPolicy};
#[doc(inline)]
pub use core::{PermissivePluginPolicy, RestrictivePluginPolicy};
