#ifdef USE_EMBER

#include <tuple>
#include <string>
#include <cstdint>

#include <Ember.h>
#include <HttpParse.h>
#include <Network.h>

#define REQUEST_BUFFER_SIZE 1024

template<typename AgentState>
class WebServer : public ember::behaviour::CyclicBehaviour<AgentState> {
  private:
    using HttpRequestParser = httpparse::HttpRequestParser;
    using HttpRequest = httpparse::Request;
    using RequestHandler = std::tuple<std::string, uint16_t>(*)(
      const HttpRequest&, 
      ember::behaviour::Context<>&, 
      AgentState&
    );

  public:
    WebServer(RequestHandler handle_request, uint16_t port)
      : handle_request_(handle_request), 
        port_(port),
        has_client_(false) {
      // Serial.print("[WebServer] Constructor called, port: ");
      // Serial.println(port_);
      server_ = NetworkServer(port_);
      server_.begin();
      // Serial.println("[WebServer] Server started");
    }

    virtual void action(
      ember::behaviour::Context<>& context,
      AgentState& agent_state
    ) override {
      // Check for new client if we don't have one
      if (!has_client_) {
        NetworkClient new_client = server_.available();
        if (new_client && new_client.connected()) {
          // Serial.println("[INFO] New client connected!");
          current_client_ = new_client;
          has_client_ = true;
          parser_ = HttpRequestParser();
          request_ = HttpRequest();
        }
        return;
      }

      // Check if current client is still connected
      if (!current_client_.connected()) {
        // Serial.println("[INFO] Client disconnected");
        has_client_ = false;
        return;
      }

      // Check for available data
      int available = current_client_.available();
      if (available <= 0) {
        return;
      }

      // Serial.print("[DEBUG] Data available: ");
      // Serial.println(available);

      // Read and parse data
      if (available > REQUEST_BUFFER_SIZE) {
        available = REQUEST_BUFFER_SIZE;
      }

      char buffer[REQUEST_BUFFER_SIZE];
      int bytes_read = current_client_.readBytes(buffer, available);

      if (bytes_read <= 0) {
        // Serial.println("[WARN] Read failed");
        cleanup_client();
        return;
      }

      // Serial.print("[DEBUG] Read ");
      // Serial.print(bytes_read);
      // Serial.println(" bytes");

      // Parse the request
      HttpRequestParser::ParseResult result = parser_.parse(
        request_,
        buffer,
        buffer + bytes_read
      );

      if (result == HttpRequestParser::ParsingCompleted) {
        // Serial.println("[INFO] Request complete!");
        // Serial.print("[DEBUG] Method: ");
        // Serial.println(request_.method.c_str());
        // Serial.print("[DEBUG] URI: ");
        // Serial.println(request_.uri.c_str());
        
        // Handle request
        std::tuple<std::string, uint16_t> response = handle_request_(
          request_, 
          context, 
          agent_state
        );
        
        std::string body = std::get<0>(response);
        uint16_t status = std::get<1>(response);
        
        // Serial.print("[DEBUG] Sending response, status: ");
        // Serial.println(status);
        
        send_response(status, body);
        cleanup_client();
        
      } else if (result == HttpRequestParser::ParsingError) {
        // Serial.println("[ERROR] Parse error");
        send_response(400, "<html><body><h1>400 Bad Request</h1></body></html>");
        cleanup_client();
      } else {
        // Serial.println("[DEBUG] Parsing incomplete, waiting for more data");
      }
    }

    virtual bool is_finished() const override {
      return false;
    }

  private:
    void send_response(uint16_t status_code, const std::string& body) {
      if (!current_client_.connected()) {
        // Serial.println("[WARN] Client disconnected, can't send");
        return;
      }

      String status_text = get_status_text(status_code);
      
      String response = "HTTP/1.1 ";
      response += String(status_code);
      response += " ";
      response += status_text;
      response += "\r\n";
      response += "Content-Type: text/html; charset=utf-8\r\n";
      response += "Content-Length: ";
      response += String(body.length());
      response += "\r\n";
      response += "Connection: close\r\n";
      response += "\r\n";
      response += body.c_str();

      // Serial.print("[DEBUG] Response size: ");
      // Serial.println(response.length());

      current_client_.print(response);
      current_client_.flush();
      
      // Serial.println("[INFO] Response sent");
    }

    String get_status_text(uint16_t status_code) {
      switch (status_code) {
        case 200: return "OK";
        case 400: return "Bad Request";
        case 404: return "Not Found";
        case 500: return "Internal Server Error";
        default: return "Unknown";
      }
    }

    void cleanup_client() {
      // Serial.println("[DEBUG] Cleaning up client");
      if (current_client_.connected()) {
        current_client_.stop();
      }
      has_client_ = false;
      parser_ = HttpRequestParser();
      request_ = HttpRequest();
    }

    RequestHandler handle_request_;
    uint16_t port_;
    NetworkServer server_;
    NetworkClient current_client_;
    HttpRequest request_;
    HttpRequestParser parser_;
    bool has_client_;
};

#endif // USE_EMBER