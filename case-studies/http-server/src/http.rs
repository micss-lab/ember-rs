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
    state: S,
}

impl<D, H, S> Server<D, H, S>
where
    D: Device,
{
    pub fn new(stack: Stack<'static, D>, port: u16, handle_request: H, state: S) -> Self {
        Self {
            stack,
            port,
            handle_request,
            state,
        }
    }
}

impl<D, H, S> CyclicBehaviour for Server<D, H, S>
where
    D: Device,
    H: FnOnce(Request, &mut S) -> (u16, String) + Clone,
{
    type AgentState = ();

    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
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

        socket.read(&mut buf).expect("failed to read from socket");

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        if let Err(err) = req.parse(&buf) {
            log::error!("Error parsing incoming request: {}", err);
        };

        log::debug!("Incoming request: {:?}", req);

        let (status, body) = self.handle_request.clone()(req, &mut self.state);
        write_response(&mut socket, status, body);

        log::trace!("Closing socket.");
        socket.flush().expect("failed to flush socket");
        socket.close();
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn write_response(mut stream: impl Write, status: u16, body: String) {
    let content_len = body.len();

    stream.write_all(b"HTTP/1.1 ").unwrap();
    stream
        .write_all(format!("{} {}\r\n", status, status_code_to_reason(status)).as_bytes())
        .unwrap();
    if content_len != 0 {
        stream
            .write_all(format!("Content-Length: {}", content_len).as_bytes())
            .unwrap();
    }
    stream.write_all(b"\r\n\r\n").unwrap();
    stream.write_all(body.as_bytes()).unwrap();
}

fn status_code_to_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        404 => "Not Found",
        _ => panic!("Status code `{}` not handled", code),
    }
}
