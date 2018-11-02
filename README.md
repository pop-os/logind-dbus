# logind-dbus

Rust crate that provides a DBus API for interacting with logind, which is useful for doing things such as inhibiting suspension.

```rust
extern crate logind_dbus;
use logind_dbus::LoginManager;

pub fn main() -> io::Result<()> {
    let login_manager = LoginManager::new()?;
    let suspend_lock = login_manager.connect().inhibit_suspend()?;
    /// Do sensitive thing with the guarantee that suspend will not work.
}
```