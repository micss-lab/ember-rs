#ifdef USE_EMBER

#include <functional>
#include <tuple>
#include <vector>
#include <cstdint>

#include <Ember.h>
#include <HttpParse.h>
#include <Network.h>

#define REQUEST_BUFFER_SIZE 1024

template<typename AgentState>
class WebServer:
    public ember::behaviour::CyclicBehaviour<AgentState> {
  private:
    using HttpRequestParser = httpparse::HttpRequestParser;
    using HttpRequest = httpparse::Request;
    using RequestHandler = std::function<
      std::tuple<std::string, uint16_t>
      (const HttpRequest&, ember::behaviour::Context<>&, AgentState&)
    >;

  public:
    WebServer(RequestHandler handle_request, uint16_t port):
      handle_request(handle_request) 
    {
      this->server = NetworkServer(port);
    }

    virtual void action(
      ember::behaviour::Context<>& context,
      AgentState& agent_state
    ) override {
      if (!this->client_set) {
        this->current_client = this->server.accept();
        this->client_set = true;
      }
      
      if (!this->current_client.connected()) {
        return;
      }

      int available = this->current_client.available();
      if (available == 0) {
        return;
      }
      if (available > REQUEST_BUFFER_SIZE) {
        available = REQUEST_BUFFER_SIZE;
      } 
      char* const buffer = new char[available];
      this->current_client.readBytes(buffer, available);

      const HttpRequestParser::ParseResult result = current_parser.parse(
        this->current_request,
        buffer,
        buffer + available
      );
      switch(result) {
        case HttpRequestParser::ParsingCompleted:
          Serial.println("[TRACE] Incoming http request.");

          // Handle the request here.
          this->handle_request(this->current_request, context, agent_state);
          
          this->current_parser = HttpRequestParser();
          this->current_request = HttpRequest();
          this->client_set = false;
          break;
        case HttpRequestParser::ParsingIncompleted:
          break;
        case HttpRequestParser::ParsingError:
          Serial.println("[ERROR] Parsing http request failed.");
          this->current_parser = HttpRequestParser();
          this->current_request = HttpRequest();
          this->client_set = false;
          break;
      }

      delete buffer;      
    }

    virtual bool is_finished() const override {
        return false;
    }

  private:
    NetworkServer server{};
    NetworkClient current_client{};
    bool client_set{false};
    HttpRequest current_request{};
    HttpRequestParser current_parser{};

    RequestHandler handle_request;
};

#endif // USE_EMBER
