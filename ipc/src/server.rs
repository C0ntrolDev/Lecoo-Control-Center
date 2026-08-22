use std::io;

use interprocess::local_socket::{Listener, ListenerOptions, prelude::*};

use crate::{DaemonWorker, get_socket_name};

pub struct IpcServer {
    listener: Listener,
}

impl IpcServer {
    /// Binds the server and sets access permissions
    pub fn bind() -> io::Result<Self> {
        let name = get_socket_name()?;
        let mut options = ListenerOptions::new().name(name);

        #[cfg(windows)]
        {
            use interprocess::os::windows::local_socket::ListenerOptionsExt;
            use interprocess::os::windows::security_descriptor::SecurityDescriptor;
            use widestring::u16cstr;

            // SDDL: SY (System) and BA (Admins) - full access (GA)
            // BU (Built-in Users) - read/write (GRGW) so that clients can connect
            let sddl = u16cstr!("D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;BU)");
            let sd = SecurityDescriptor::deserialize(sddl)?;
            options = options.security_descriptor(sd);
        }

        #[cfg(unix)]
        {
            use interprocess::os::unix::local_socket::ListenerOptionsExt;
            // rw-rw-rw-
            options = options.mode(0o666);
        }

        let listener = options.create_sync()?;
        Ok(Self { listener })
    }

    /// Iterator over incoming connections
    pub fn accept(&mut self) -> io::Result<DaemonWorker> {
        let stream = self.listener.accept()?;
        Ok(DaemonWorker::new(stream))
    }
}
