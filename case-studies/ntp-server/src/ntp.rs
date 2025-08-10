use core::cell::RefCell;

use blocking_network_stack::{Stack, UdpSocket};
use no_std_framework_core::behaviour::{Context, TickerBehaviour};
use smoltcp::phy::Device;
use sntpc::{NtpContext, NtpTimestampGenerator, NtpUdpSocket};

use super::wrapper::W;

pub struct Server<D>
where
    D: Device,
{
    stack: Stack<'static, D>,
}

impl<D> Server<D>
where
    D: Device,
{
    pub fn new(stack: Stack<'static, D>) -> Self {
        Self { stack }
    }
}

impl<D> TickerBehaviour for Server<D>
where
    D: Device,
{
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(2)
    }

    fn action(&mut self, _ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let (mut rx_buffer, mut rx_metadata) = (
            [0u8; 2048],
            [smoltcp::socket::udp::PacketMetadata::EMPTY; 4],
        );
        let (mut tx_buffer, mut tx_metadata) = (
            [0u8; 1024],
            [smoltcp::socket::udp::PacketMetadata::EMPTY; 4],
        );
        let mut socket = self.stack.get_udp_socket(
            &mut rx_metadata,
            &mut rx_buffer,
            &mut tx_metadata,
            &mut tx_buffer,
        );
        // Bind socket on some random port to avoid `Unaddressable` errors when sending data.
        socket.bind(9000).unwrap();

        let time = {
            let time = await_future(sntpc::get_time(
                ([80, 209, 87, 103], 123).into(),
                &W(RefCell::new(socket)),
                NtpContext::new(TimestampGenerator),
            ))
            .expect("failed to fetch time from ntp server");
            chrono::DateTime::from_timestamp(time.seconds as i64, 0).unwrap()
        };

        log::info!("Current time: {}", time.format("%Y-%m-%d %H:%M:%S"))
    }

    fn is_finished(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy)]
struct TimestampGenerator;

impl NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 {
        0
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        0
    }
}

impl<D> NtpUdpSocket for W<RefCell<UdpSocket<'_, '_, '_, D>>>
where
    D: Device,
{
    fn send_to(
        &self,
        buf: &[u8],
        addr: core::net::SocketAddr,
    ) -> impl core::future::Future<Output = sntpc::Result<usize>> {
        let mut socket = RefCell::borrow_mut(self);
        let len = buf.len();

        socket
            .send(core_to_smoltcp_wire(addr.ip()), addr.port(), buf)
            .expect("failed to send data over udp socket");
        core::future::ready(Ok(len))
    }

    fn recv_from(
        &self,
        buf: &mut [u8],
    ) -> impl core::future::Future<Output = sntpc::Result<(usize, core::net::SocketAddr)>> {
        use blocking_network_stack::IoError;
        use smoltcp::socket::udp::RecvError;

        let mut socket = RefCell::borrow_mut(self);
        let (len, ip_addr, port) = loop {
            match socket.receive(buf) {
                Ok(r) => break r,
                Err(IoError::UdpRecvError(RecvError::Exhausted)) => (),
                Err(err) => panic!("failed to receive data over udp socket: {:?}", err),
            }
        };

        core::future::ready(Ok((
            len,
            core::net::SocketAddr::from((smoltcp_wire_to_core(ip_addr), port)),
        )))
    }
}

fn core_to_smoltcp_wire(addr: core::net::IpAddr) -> smoltcp::wire::IpAddress {
    use core::net::IpAddr;
    use smoltcp::wire::Ipv4Address as SmoltcpIpv4Addr;

    match addr {
        IpAddr::V4(addr) => {
            let [a, b, c, d] = addr.octets();
            SmoltcpIpv4Addr::new(a, b, c, d).into()
        }
        IpAddr::V6(_) => unimplemented!("sending to an ipv6 address is not supported"),
    }
}

fn smoltcp_wire_to_core(addr: smoltcp::wire::IpAddress) -> core::net::IpAddr {
    use core::net::Ipv4Addr;
    use smoltcp::wire::IpAddress as SmoltcpIpAddr;

    match addr {
        SmoltcpIpAddr::Ipv4(addr) => Ipv4Addr::from(addr.octets()).into(),
    }
}

fn await_future<F>(fut: F) -> F::Output
where
    F: core::future::Future,
{
    use core::{pin::pin, task::Context};
    use futures::task::noop_waker;

    let waker = noop_waker();
    let mut ctx = Context::from_waker(&waker);

    let mut fut = pin!(fut);
    loop {
        match fut.as_mut().poll(&mut ctx) {
            core::task::Poll::Ready(r) => break r,
            core::task::Poll::Pending => (),
        }
    }
}
