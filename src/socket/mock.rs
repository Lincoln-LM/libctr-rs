use super::{SocketAddr, SocketType};
use crate::res::CtrResult;

use core::cell::RefCell;

pub struct InternalSocketContext {
    pub socket_type: SocketType,
    pub socket_buffer_size: u32,
}

pub struct SocketContext {
    pub mock: RefCell<InternalSocketContext>,
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl SocketContext {
    pub fn new(socket_type: SocketType) -> CtrResult<Self> {
        let context = InternalSocketContext {
            socket_type,
            socket_buffer_size: 0,
        };
        Ok(Self {
            mock: RefCell::new(context),
        })
    }

    pub fn connect(&self, address: SocketAddr) -> CtrResult {
        Ok(())
    }

    pub fn bind(&self, address: SocketAddr) -> CtrResult {
        Ok(())
    }

    pub fn listen(&self, max_connections: u32) -> CtrResult {
        Ok(())
    }

    pub fn accept(&self) -> CtrResult<SocketAddr> {
        Ok(SocketAddr::new([0, 0, 0, 0], 0))
    }

    pub fn recv(&self, result_buffer: &mut [u8], flags: u32) -> CtrResult {
        Ok(())
    }

    pub fn send(&self, send_buffer: &[u8], flags: u32) -> CtrResult {
        Ok(())
    }

    pub fn get_host_id(&self) -> CtrResult<[u8; 4]> {
        Ok([0, 0, 0, 0])
    }
}
