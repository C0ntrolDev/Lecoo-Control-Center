use crate::{DaemonClient, IpcRequest, IpcResponse, get_socket_name};

pub struct IpcClient {
    conn: DaemonClient,
}

impl IpcClient {
    pub fn connect() -> std::io::Result<Self> {
        let name = get_socket_name()?;
        let stream = interprocess::local_socket::ConnectOptions::new()
            .name(name.borrow())
            .connect_sync()?;

        Ok(Self {
            conn: DaemonClient::connect_client_handshake(stream)?,
            // daemon_version: (0, 0) // todo: now we don't know the daemon version
        })
    }

    pub fn request(&mut self, req: &IpcRequest) -> std::io::Result<IpcResponse> {
        self.conn.send(req)?;
        self.conn.recv()
    }
}
