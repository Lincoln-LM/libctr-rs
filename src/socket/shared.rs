#[cfg(target_os = "horizon")]
use crate::ipc::{get_thread_static_buffers, make_static_buffer_header};
use crate::{
    ipc::ThreadCommandBuilder, memory::MemoryBlock, res::CtrResult, srv::get_service_handle_direct,
    utils::convert::try_usize_into_u32, Handle,
};
#[cfg(target_os = "horizon")]
use core::convert::TryInto;
use core::sync::atomic::{AtomicU32, Ordering};
use num_enum::IntoPrimitive;

#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive)]
#[repr(u32)]
pub enum SocketType {
    Stream = 1,
    Dgram = 2,
}

pub struct SocketAddr {
    pub size: u8,
    pub family: u8,
    pub address: [u8; 4],
    pub port: u16,
}

impl SocketAddr {
    pub fn new(address: [u8; 4], port: u16) -> Self {
        SocketAddr {
            // 0x1c for AF_INET6, unimplemented
            size: 0x8,
            // 23 for AF_INET6, unimplemented
            family: 2,
            address,
            port,
        }
    }
}

static SOCU_SERVICE_HANDLE: AtomicU32 = AtomicU32::new(0);
static SOCU_SHARED_MEMORY_HANDLE: AtomicU32 = AtomicU32::new(0);

pub(crate) fn get_socu_service_raw_handle() -> u32 {
    SOCU_SERVICE_HANDLE.load(Ordering::Relaxed)
}

pub fn socu_init(memory_block: MemoryBlock) -> CtrResult {
    let service_handle = get_service_handle_direct("soc:U")?;

    socu_initialize_sockets(
        &service_handle,
        memory_block.get_size(),
        memory_block.get_handle(),
    )?;

    // We need to drop this since the raw handles are stored in an atomic
    let dropped_service_handle = core::mem::ManuallyDrop::new(service_handle);
    let dropped_memory_block = core::mem::ManuallyDrop::new(memory_block);

    // This is safe since we're sending it to another process, not copying it
    let raw_service_handle = unsafe { dropped_service_handle.get_raw() };
    SOCU_SERVICE_HANDLE.store(raw_service_handle, Ordering::Relaxed);

    // This is safe since we're sending it to another process, not copying it
    let shared_memory_raw_handle = unsafe { dropped_memory_block.get_handle().get_raw() };
    SOCU_SHARED_MEMORY_HANDLE.store(shared_memory_raw_handle, Ordering::Relaxed);

    Ok(())
}

fn socu_initialize_sockets(
    service_handle: &Handle,
    shared_memory_block_size: usize,
    shared_memory_block_handle: &Handle,
) -> CtrResult {
    let shared_memory_block_size = try_usize_into_u32(shared_memory_block_size)?;

    let mut command = ThreadCommandBuilder::new(0x1u16);
    command.push(shared_memory_block_size);
    command.push_curent_process_id();
    command.push_shared_handles(&[shared_memory_block_handle])?;

    let mut parser = command.build().send_sync_request(service_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_socket(socket_type: SocketType) -> CtrResult<SocketContextHandle> {
    let mut command = ThreadCommandBuilder::new(0x2u16);
    // Domain must always be 2 (AF_INET)
    command.push(2u32);
    command.push(socket_type);
    // Protocol must always be 0
    command.push(0u32);
    command.push_curent_process_id();

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_socu_service_raw_handle())?;
    parser.pop_result()?;

    Ok(parser.pop().into())
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_connect(
    session_handle: &Handle,
    context_handle: &SocketContextHandle,
    socket_addr: SocketAddr,
) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0x6u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    // TODO: convert struct to bytes directly?
    let buffer: [u8; 8] = [
        socket_addr.size,
        socket_addr.family,
        (socket_addr.port >> 8).try_into().unwrap(),
        (socket_addr.port & 0xFF).try_into().unwrap(),
        socket_addr.address[0],
        socket_addr.address[1],
        socket_addr.address[2],
        socket_addr.address[3],
    ];
    command.push(socket_addr.size as u32);
    command.push_curent_process_id();
    command.push_static_buffer(&buffer, 0);

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;
    let posix_result = parser.pop().into();
    if posix_result == 0 {
        Ok(())
    } else {
        Err(posix_result)
    }
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_bind(
    session_handle: &Handle,
    context_handle: &SocketContextHandle,
    socket_addr: SocketAddr,
) -> CtrResult {
    // TODO: convert struct to bytes directly?
    let buffer: [u8; 8] = [
        socket_addr.size,
        socket_addr.family,
        (socket_addr.port >> 8).try_into().unwrap(),
        (socket_addr.port & 0xFF).try_into().unwrap(),
        socket_addr.address[0],
        socket_addr.address[1],
        socket_addr.address[2],
        socket_addr.address[3],
    ];
    let mut command = ThreadCommandBuilder::new(0x5u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(socket_addr.size as u32);
    command.push_curent_process_id();
    command.push_static_buffer(&buffer, 0);

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;
    let posix_result = parser.pop().into();
    if posix_result == 0 {
        Ok(())
    } else {
        Err(posix_result)
    }
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_get_host_id(session_handle: &Handle) -> CtrResult<u32> {
    let command = ThreadCommandBuilder::new(0x16u16);
    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;
    Ok(parser.pop().into())
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_listen(
    session_handle: &Handle,
    context_handle: &SocketContextHandle,
    max_connections: u32,
) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0x3u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(max_connections);
    command.push_curent_process_id();

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;
    let posix_result = parser.pop().into();
    if posix_result == 0 {
        Ok(())
    } else {
        Err(posix_result)
    }
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_accept(
    session_handle: &Handle,
    context_handle: &SocketContextHandle,
    client_socket_addr: &mut SocketAddr,
) -> CtrResult<u32> {
    // TODO: convert struct to bytes directly?
    let buffer: [u8; 8] = [
        client_socket_addr.size,
        client_socket_addr.family,
        (client_socket_addr.port >> 8).try_into().unwrap(),
        (client_socket_addr.port & 0xFF).try_into().unwrap(),
        client_socket_addr.address[0],
        client_socket_addr.address[1],
        client_socket_addr.address[2],
        client_socket_addr.address[3],
    ];
    let mut saved_thread_storage = [0u32; 2];
    let mut command = ThreadCommandBuilder::new(0x4u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    // 0x1c to account for IPv6 which isnt implemented
    command.push(0x1Cu32);
    command.push_curent_process_id();

    let static_buffers = get_thread_static_buffers();
    saved_thread_storage[0] = static_buffers[0];
    saved_thread_storage[1] = static_buffers[1];

    static_buffers[0] = make_static_buffer_header(0x1c, 0);
    static_buffers[1] = buffer.as_ptr() as u32;

    let mut parser = command.build().send_sync_request(session_handle)?;

    static_buffers[0] = saved_thread_storage[0];
    static_buffers[1] = saved_thread_storage[1];

    client_socket_addr.port = ((buffer[2] as u16) << 8u16) | (buffer[3] as u16);
    client_socket_addr.address[0] = buffer[4];
    client_socket_addr.address[1] = buffer[5];
    client_socket_addr.address[2] = buffer[6];
    client_socket_addr.address[3] = buffer[7];
    parser.pop_result()?;
    Ok(parser.pop() as u32)
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_recv(
    session_handle: &Handle,
    context_handle: &SocketContextHandle,
    result_buffer: &mut [u8],
    flags: u32,
) -> CtrResult {
    let buffer = [0u8; 0x1c];

    let mut saved_thread_storage = [0u32; 2];
    let mut command = ThreadCommandBuilder::new(0x7u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(try_usize_into_u32(result_buffer.len()).unwrap());
    command.push(flags);
    // If we wanted to return src_addr, this would be 0x1c
    command.push(0u32);
    command.push_curent_process_id();
    command.push_write_buffer(result_buffer);

    let static_buffers = get_thread_static_buffers();
    saved_thread_storage[0] = static_buffers[0];
    saved_thread_storage[1] = static_buffers[1];

    // If we wanted to return src_addr, size would be 0x1c
    static_buffers[0] = make_static_buffer_header(0, 0);
    static_buffers[1] = buffer.as_ptr() as u32;

    let mut parser = command.build().send_sync_request(session_handle)?;

    static_buffers[0] = saved_thread_storage[0];
    static_buffers[1] = saved_thread_storage[1];

    parser.pop_result()?;
    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn socu_send(
    session_handle: &Handle,
    context_handle: &SocketContextHandle,
    send_buffer: &[u8],
    flags: u32,
) -> CtrResult {
    let buffer = [0u8; 0x1c];

    let mut command = ThreadCommandBuilder::new(0x9u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(try_usize_into_u32(send_buffer.len()).unwrap());
    command.push(flags);
    // If we wanted to return src_addr, this would be 0x1c
    command.push(0u32);
    command.push_curent_process_id();
    command.push_static_buffer(&buffer, 1);
    command.push_read_buffer(send_buffer);

    let mut parser = command.build().send_sync_request(session_handle)?;

    parser.pop_result()?;
    Ok(())
}

pub(crate) struct SocketContextHandle(u32);

#[cfg(target_os = "horizon")]
impl SocketContextHandle {
    /// Returns the raw u32 handle
    /// # Safety
    /// Because a Handle closes itself when it's dropped, a raw handle might have been previously closed.
    /// The user must guarantee the handle will outlive the raw handle (and all copies/clones of the raw handle)
    ///
    /// Admittedly this is less of memory safety and more of logical safety, but since that's the purpose of this abstraction
    /// unsafe will be used in this way.
    pub(crate) unsafe fn get_raw(&self) -> u32 {
        self.0
    }
}

impl From<u32> for SocketContextHandle {
    fn from(raw_handle: u32) -> Self {
        Self(raw_handle)
    }
}

impl Drop for SocketContextHandle {
    // If this doesn't close, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        let mut command = ThreadCommandBuilder::new(0xBu16);
        command.push(self.0);
        command.push_curent_process_id();
        command
            .build()
            .send_sync_request_with_raw_handle(get_socu_service_raw_handle());
    }
}
