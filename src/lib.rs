//! Provides a DBus API for interacting with logind, which is useful for doing things such as inhibiting suspension.
//!
//! ```rust,no_run
//! extern crate logind_dbus;
//! use logind_dbus::LoginManager;
//! 
//! pub fn main() -> io::Result<()> {
//!     let login_manager = LoginManager::new()?;
//!     let suspend_lock = login_manager.connect().inhibit_suspend()?;
//!     /// Do sensitive thing with the guarantee that suspend will not work.
//! }
//! ```

#[macro_use]
extern crate cascade;
extern crate dbus;

use dbus::{arg, BusType, Connection, ConnPath};
use std::ops::Deref;

/// An interface to `org.freedesktop.login1.Manager`.
pub struct LoginManager {
    conn: Connection
}

impl Deref for LoginManager {
    type Target = Connection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl LoginManager {
    pub fn new() -> Result<LoginManager, dbus::Error> {
        Ok(Self { conn: Connection::get_private(BusType::System)? })
    }

    pub fn connect(&self) -> LoginManagerConnection {
        LoginManagerConnection {
            conn: self.with_path("org.freedesktop.login1", "/org/freedesktop/login1", 1000)
        }
    }
}

/// An established connection path for the login manager, through which the API is made accessible.
pub struct LoginManagerConnection<'a> {
    conn: ConnPath<'a, &'a Connection>
}

impl<'a> LoginManagerConnection<'a> {
    /// Inhibit is the only API necessary to take a lock. It takes four arguments:
    /// 
    /// - **What** is a colon-separated list of lock types, i.e. `shutdown`, `sleep`, `idle`,
    ///   `handle-power-key`, `handle-suspend-key`, `handle-hibernate-key`, `handle-lid-switch`.
    ///   Example: "shutdown:idle"
    /// - **Who** is a human-readable, descriptive string of who is taking the lock. Example: "Package Updater"
    /// - **Why** is a human-readable, descriptive string of why the lock is taken. Example: "Package Update in Progress"
    /// - **Mode** is one of `block` or `delay`.
    /// 
    /// # Notes
    /// 
    /// A root user session cannot use systemd inhibitors.
    pub fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> Result<dbus::OwnedFd, dbus::Error> {
        let mut m = self.conn.method_call_with_args(
            &"org.freedesktop.login1.Manager".into(),
            &"Inhibit".into(),
            |msg| {
                cascade! {
                    arg::IterAppend::new(msg);
                    ..append(what);
                    ..append(who);
                    ..append(why);
                    ..append(mode);
                }
            })?;

        m.as_result()?;
        Ok(m.iter_init().read::<dbus::OwnedFd>()?)
    }

    /// Convenience method for inhibiting suspend.
    /// 
    /// Equivalent to `connection.inhibit("idle:shutdown:sleep", who, why, "block")`.
    pub fn inhibit_suspend(&self, who: &str, why: &str) -> Result<dbus::OwnedFd, dbus::Error> {
        self.inhibit("idle:shutdown:sleep", who, why, "block")
    }
}
