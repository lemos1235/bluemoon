mod clash;
#[allow(clippy::module_inception)]
mod config;
mod draft;
mod prfitem;
mod profiles;
mod runtime;
mod verge;
mod moon;

pub use self::clash::*;
pub use self::config::*;
pub use self::draft::*;
pub use self::prfitem::*;
pub use self::profiles::*;
pub use self::runtime::*;
pub use self::verge::*;
pub use self::moon::*;

pub const DEFAULT_PAC: &str = r#"function FindProxyForURL(url, host) {
  return "PROXY 127.0.0.1:%mixed-port%; SOCKS5 127.0.0.1:%mixed-port%; DIRECT;";
}
"#;
