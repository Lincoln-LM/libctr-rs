use super::{
    socu_accept, socu_bind, socu_connect, socu_get_host_id, socu_listen, socu_recv, socu_send,
    socu_socket, SocketAddr, SocketContextHandle, SocketType,
};
use crate::{res::CtrResult, srv::get_service_handle_direct, Handle};

pub struct SocketContext {
    session_handle: Handle,
    context_handle: SocketContextHandle,
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl SocketContext {
    pub fn new(socket_type: SocketType) -> CtrResult<Self> {
        let context_handle = socu_socket(socket_type)?;
        let session_handle = get_service_handle_direct("soc:U")?;

        Ok(Self {
            session_handle,
            context_handle,
        })
    }

    pub fn connect(&self, address: SocketAddr) -> CtrResult {
        socu_connect(&self.session_handle, &self.context_handle, address)
    }

    pub fn bind(&self, address: SocketAddr) -> CtrResult {
        socu_bind(&self.session_handle, &self.context_handle, address)
    }

    pub fn listen(&self, max_connections: u32) -> CtrResult {
        socu_listen(&self.session_handle, &self.context_handle, max_connections)
    }

    pub fn accept(&mut self) -> CtrResult<SocketAddr> {
        let mut client_socket_addr = SocketAddr::new([0, 0, 0, 0], 0);
        let handle = socu_accept(
            &self.session_handle,
            &self.context_handle,
            &mut client_socket_addr,
        )?;
        // Update handle for the new connection
        self.context_handle = handle.into();
        Ok(client_socket_addr)
    }

    pub fn recv(&self, result_buffer: &mut [u8], flags: u32) -> CtrResult {
        socu_recv(
            &self.session_handle,
            &self.context_handle,
            result_buffer,
            flags,
        )
    }

    pub fn send(&self, send_buffer: &[u8], flags: u32) -> CtrResult {
        socu_send(
            &self.session_handle,
            &self.context_handle,
            send_buffer,
            flags,
        )
    }

    pub fn get_host_id(&self) -> CtrResult<[u8; 4]> {
        Ok(socu_get_host_id(&self.session_handle)?.to_le_bytes())
    }
}
