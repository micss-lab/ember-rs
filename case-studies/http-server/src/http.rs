use alloc::format;
use alloc::string::String;

use blocking_network_stack::Stack;

use embedded_io::Write;
use httparse::Request;
use no_std_framework_core::behaviour::{Context, CyclicBehaviour};
use smoltcp::phy::Device;

mod routes {
    use alloc::format;
    use alloc::string::{String, ToString};

    #[derive(Default)]
    pub(super) enum LedState {
        On,
        #[default]
        Off,
    }

    impl core::fmt::Display for LedState {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::On => "on",
                    Self::Off => "off",
                }
            )
        }
    }

    impl LedState {
        pub(super) fn toggle(&mut self) {
            *self = match self {
                Self::On => Self::Off,
                Self::Off => Self::On,
            }
        }
    }

    #[derive(Default)]
    pub(super) struct State {
        pub(super) led1: LedState,
        pub(super) led2: LedState,
    }

    pub(super) fn index(state: &State) -> String {
        format!(
            r#"
                <!DOCTYPE html><html>
                  <head>
                    <title>ESP32 Web Server Demo</title>
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                    <style>
                      html {{ font-family: sans-serif; text-align: center; }}
                      body {{ display: inline-flex; flex-direction: column; }}
                      h1 {{ margin-bottom: 1.2em; }}
                      h2 {{ margin: 0; }}
                      div {{ display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: auto auto; grid-auto-flow: column; grid-gap: 1em; }}
                      .btn {{ background-color: #5B5; border: none; color: #fff; padding: 0.5em 1em; font-size: 2em; text-decoration: none }}
                      .btn.off {{ background-color: #333; }}
                    </style>
                  </head>

                  <body>
                    <h1>ESP32 Web Server</h1>

                    <div>
                      <h2>LED 1</h2>
                      <a href="/toggle/1" class="btn {}">{}</a>
                      <h2>LED 2</h2>
                      <a href="/toggle/2" class="btn {}">{}</a>
                    </div>
                  </body>
                </html>
            "#,
            state.led1,
            state.led1.to_string().to_uppercase(),
            state.led2,
            state.led2.to_string().to_uppercase(),
        )
    }

    pub(super) fn not_found(method: &str, path: &str) -> String {
        format!("Not found: path `{}`, method: `{}`", path, method)
    }
}

pub struct Server<D>
where
    D: Device,
{
    stack: Stack<'static, D>,
    port: u16,
    state: routes::State,
}

impl<D> Server<D>
where
    D: Device,
{
    pub fn new(stack: Stack<'static, D>, port: u16) -> Self {
        Self {
            stack,
            port,
            state: routes::State::default(),
        }
    }
}

impl<D> CyclicBehaviour for Server<D>
where
    D: Device,
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

        let (status, body) = handle_request(req, &mut self.state);
        write_response(&mut socket, status, body);

        log::trace!("Closing socket.");
        socket.flush().expect("failed to flush socket");
        socket.close();
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn handle_request(req: Request, state: &mut routes::State) -> (u16, String) {
    match (req.method.unwrap_or("GET"), req.path.unwrap_or("/")) {
        ("GET", "/") => (200, routes::index(state)),
        ("GET", path) if path.starts_with("/toggle/") => {
            let (_, led) = path.split_once("/toggle/").unwrap();
            match led.parse() {
                Ok(1) => state.led1.toggle(),
                Ok(2) => state.led2.toggle(),
                Ok(idx) => return (400, format!("Unknown led index {idx}")),
                Err(err) => return (400, format!("Invalid led index {led}: {}", err)),
            }
            (200, routes::index(state))
        }
        (method, path) => (404, routes::not_found(method, path)),
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
