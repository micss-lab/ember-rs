use alloc::format;
use alloc::string::String;
use ember::behaviour::Context;

use httparse::Request;

use super::HomeData;

fn index(state: &HomeData) -> String {
    format!(
        r#"
        <!DOCTYPE html><html>
          <head>
            <title>ESP32 Smart Home Dashboard</title>
            <meta name="viewport" content="width=device-width, initial-scale=1">
          </head>

          <body>
            <h1>ESP32 Smart Home Dashboard</h1>

            <table border="1" cellpadding="5" cellspacing="0">
              <tr>
                <th>Field</th>
                <th>Value</th>
              </tr>
              <tr>
                <td>Moisture</td>
                <td>{}</td>
              </tr>
              <tr>
                <td>Light Level</td>
                <td>{}</td>
              </tr>
              <tr>
                <td>Temperature</td>
                <td>{}</td>
              </tr>
              <tr>
                <td>Pump Active</td>
                <td>{}</td>
              </tr>
              <tr>
                <td>Door Locked</td>
                <td>{}</td>
              </tr>
              <tr>
                <td>Fan Active</td>
                <td>{}</td>
              </tr>
              <tr>
                <td>Human Home</td>
                <td>{}</td>
              </tr>
            </table>
          </body>
        </html>
    "#,
        state.moisture,
        state.light_level,
        state.temperature,
        state.pump_active,
        state.door_locked,
        state.fan_active,
        state.human_home,
    )
}

fn not_found(method: &str, path: &str) -> String {
    format!("Not found: path `{}`, method: `{}`", path, method)
}

pub fn handle_request(req: Request, _: &mut Context<()>, state: &mut HomeData) -> (u16, String) {
    match (req.method.unwrap_or("GET"), req.path.unwrap_or("/")) {
        ("GET", "/") => (200, index(state)),
        (method, path) => (404, not_found(method, path)),
    }
}
