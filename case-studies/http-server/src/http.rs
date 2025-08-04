use core::marker::PhantomData;

use alloc::format;
use alloc::string::String;

use blocking_network_stack::Stack;

use embedded_io::Write;
use httparse::Request;
use no_std_framework_core::behaviour::{Context, CyclicBehaviour};
use smoltcp::phy::Device;

pub struct Server<D, H, S>
where
    D: Device,
{
    stack: Stack<'static, D>,
    port: u16,
    handle_request: H,
    _state: PhantomData<S>,
}

impl<D, H, S> Server<D, H, S>
where
    D: Device,
    // Signature added here for better compiler errors.
    H: FnOnce(Request, &mut Context<()>, &mut S) -> (u16, String) + Clone,
{
    pub fn new(stack: Stack<'static, D>, port: u16, handle_request: H) -> Self {
        Self {
            stack,
            port,
            handle_request,
            _state: PhantomData,
        }
    }
}

impl<D, H, S> CyclicBehaviour for Server<D, H, S>
where
    D: Device,
    H: FnOnce(Request, &mut Context<()>, &mut S) -> (u16, String) + Clone,
{
    type AgentState = S;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        use embedded_io::Read;

        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 2048];
        let mut socket = self.stack.get_socket(&mut rx_buffer, &mut tx_buffer);
        if let Err(err) = socket.listen(self.port) {
            log::error!("Error listening for incoming connection: {:?}", err);
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
            log::error!("Error parsing incoming request: {}", err);
        };

        log::debug!("Incoming request: {:?}", req);

        let (status, body) = self.handle_request.clone()(req, ctx, state);
        if let Err(err) = write_response(&mut socket, status, body) {
            log::warn!("failed to send response: {:?}", err);
        }

        log::trace!("Closing socket.");
        let _ = socket.flush();
        socket.close();
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
        stream.write_all(format!("Content-Length: {}", content_len).as_bytes())?;
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
