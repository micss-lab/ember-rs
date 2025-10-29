#define USE_EMBER // Uncomment this to use the ember library.

#include <array>
#include <vector>

#include "common.h"

#ifdef USE_EMBER

#include "Ember.h"

#include "./agents/SortAgent.h"
#include "./agents/BuildAgent.h"
#include "./agents/TrashAgent.h"

#endif // USE_EMBER

const std::array<Colour, 13> sequence = {
  Colour::Blue,
  Colour::Red,
  Colour::Red,
  Colour::Green,
  Colour::Green,
  Colour::Blue,
  Colour::Green,
  Colour::Red,
  Colour::Blue,
  Colour::Red,
  Colour::Blue,
  Colour::Blue,
  Colour::Blue,
};

#ifdef USE_EMBER

std::unique_ptr<ember::Container> container;

#else 



#endif // USE_EMBER

/******************************************
  Setup
******************************************/
void setup() {
  Serial.begin(115200);

  #ifdef USE_EMBER

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  auto belt = std::make_shared<Belt>(std::begin(sequence), std::end(sequence));
  auto sort_agent = agents::sort::create_sort_agent(belt);
  auto trash_agent = agents::trasher::create_trasher_agent(belt);
  auto build_agent = agents::builder::create_builder_agent(belt);

  container->add_agent(std::move(sort_agent));
  container->add_agent(std::move(trash_agent));
  container->add_agent(std::move(build_agent));

  #endif // USE_EMBER

  Serial.println("Colour Combinator System Initialized");
}

/******************************************
  Loop
******************************************/
void loop() {
  #ifdef USE_EMBER

  ember::Container::PollResult result = container->poll();
  if (result.should_stop) {
    Serial.println(
      (result.status == 0) ? "Finished executing." : "Container exited with an error!"
    );
    exit(result.status);
  }

  #else

  // Create sequence of optional colours, with None at the end
  std::vector<std::optional<Colour>> opt_sequence;
  
  for (Colour c : sequence) {
      opt_sequence.push_back(std::optional(std::move(c)));
  }
  opt_sequence.push_back(std::nullopt);
  
  size_t score = 0;
  std::optional<Colour> stored = std::nullopt;
  
  // Process sliding windows of size 2 
  // (generated from case-studies/colour-combinations/src/entry.rs)
  for (size_t i = 0; i + 1 < opt_sequence.size(); ++i) {
      const auto& window0 = opt_sequence[i];
      const auto& window1 = opt_sequence[i + 1];
      
      // Match pattern: (stored, window[0], window[1])
      if (window0 && window0 == Colour::Red) {
          // (s, Some(Red), _)
          if (stored) {
              score += combine_colours(stored.value(), Colour::Red);
              stored = std::nullopt;
          } else {
              stored = Colour::Red;
          }
      }
      else if (stored && window0) {
          // (Some(s), Some(c1), _)
          score += combine_colours(stored.value(), *window0);
          stored = std::nullopt;
      }
      else if (!stored && window0 && window1 && window1 == Colour::Red) {
          // (None, Some(_), Some(Red))
          stored = Colour::Red;
          ++i; // Skip next window
      }
      else if (!stored && window0 && window1 && *window0 == *window1) {
          // (None, Some(c1), Some(c2)) where c1 == c2
          score += combine_colours(window0.value(), *window1);
          ++i; // Skip next window
      }
      else if (!stored && window0 && window1 && *window0 != *window1) {
          // (None, Some(c1), Some(c2)) where c1 != c2
          stored = *window1;
          ++i; // Skip next window
      }
  }

  // Stop looping
  while (true) {}
  
  #endif // USE_EMBER
}