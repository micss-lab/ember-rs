use core::{marker::PhantomData, ptr::addr_of_mut};

use alloc::format;
use alloc::string::String;

use blocking_network_stack::Socket;

use embedded_io::Write;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use httparse::Request;
use ember::behaviour::{Context, CyclicBehaviour};
use smoltcp::phy::Device;

static mut RX_BUFFER: &mut [u8] = &mut [0u8; 1024];
static mut TX_BUFFER: &mut [u8] = &mut [0u8; 2048];

pub struct Server<H, S, D>
where
    D: Device + 'static,
{
    port: u16,
    handle_request: H,
    current_socket: Option<Socket<'static, 'static, 'static, D>>,
    _state: PhantomData<S>,
}

impl<H, S, D> Server<H, S, D>
where
    D: Device + 'static,
    // Signature added here for better compiler errors.
    H: FnOnce(Request, &mut Context<()>, &mut S) -> (u16, String) + Clone,
{
    pub fn new(port: u16, handle_request: H) -> Self {
        Self {
            port,
            handle_request,
            current_socket: None,
            _state: PhantomData,
        }
    }
}

impl<H, S> CyclicBehaviour for Server<H, S, WifiDevice<'static, WifiStaDevice>>
where
    H: FnOnce(Request, &mut Context<()>, &mut S) -> (u16, String) + Clone,
{
    type AgentState = S;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        use embedded_io::Read;

        let mut socket = self.current_socket.take().unwrap_or_else(|| {
            let mut socket = unsafe { &mut *addr_of_mut!(crate::WIFI_STACK) }
                .get_mut()
                .unwrap()
                .get_socket(unsafe { *addr_of_mut!(RX_BUFFER) }, unsafe {
                    *addr_of_mut!(TX_BUFFER)
                });
            socket.listen_unblocking(self.port).unwrap();
            socket
        });

        if !socket.is_connected() {
            socket.work();
            self.current_socket = Some(socket);
            return;
        }

        log::trace!("Incoming connection.");

        let mut buf = [0u8; 1024];

        if socket.read(&mut buf).is_err() {
            return;
        }

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        if let Err(err) = req.parse(&buf) {
            log::error!("Error parsing incoming request: {err}");
        };

        log::debug!("Incoming request: {req:?}");

        let (status, body) = self.handle_request.clone()(req, ctx, state);
        if let Err(err) = write_response(&mut socket, status, body) {
            log::warn!("failed to send response: {err:?}");
        }

        log::trace!("Closing socket.");
        if let Err(err) = socket.flush() {
            log::error!("Error closing socket: {err:?}");
        }
        socket.close();
        log::debug!("Successfully sent response and closed socket.");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn write_response<W>(mut stream: W, status: u16, body: String) -> Result<(), W::Error>
where
    W: Write,
{
    let content_len = body.len();

    stream.write_all(b"HTTP/1.1 ")?;
    stream.write_all(format!("{} {}\r\n", status, status_code_to_reason(status)).as_bytes())?;
    if content_len != 0 {
        stream.write_all(format!("Content-Length: {content_len}").as_bytes())?;
    }
    stream.write_all(b"\r\n\r\n")?;
    stream.write_all(body.as_bytes())?;
    Ok(())
}

fn status_code_to_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        404 => "Not Found",
        _ => panic!("Status code `{}` not handled", code),
    }
}
