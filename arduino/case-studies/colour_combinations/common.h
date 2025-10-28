#include <deque>
#include <optional>
#include <iostream>

enum class Colour {
    Red,
    Green,
    Blue,
};

unsigned int combine_colours(Colour bottom, Colour top) {
    if (bottom == Colour::Red && top == Colour::Red) {
        return 100;
    }
    if (bottom == Colour::Red || top == Colour::Red) {
        return 50;
    }
    if ((bottom == Colour::Green && top == Colour::Green) ||
        (bottom == Colour::Blue && top == Colour::Blue)) {
        return 25;
    }
    return 0;
}

struct Window {
    Colour first;
    std::optional<Colour> second;
    
    Window(Colour f, std::optional<Colour> s) 
        : first(f), second(s) {}
};

class Belt {
private:
    std::deque<Colour> items;
    size_t score;

public:
    template<typename Iterator>
    Belt(Iterator begin, Iterator end) 
        : items(begin, end), score(0) {}
    
    Belt(std::initializer_list<Colour> init)
        : items(init), score(0) {}

    std::optional<Colour> take_next() {
        if (items.empty()) {
            return std::nullopt;
        }
        Colour front = items.front();
        items.pop_front();
        return front;
    }

    std::optional<Colour> peek_next() const {
        if (items.empty()) {
            return std::nullopt;
        }
        return items.front();
    }

    std::optional<Window> next_window() {
        auto first = take_next();
        if (!first) {
            return std::nullopt;
        }
        
        auto second = peek_next();
        items.push_front(*first);
        return Window(*first, second);
    }

    unsigned int made_combination(const Colour& bottom, const Colour& top) {
        unsigned int val = combine_colours(bottom, top);
        score += val;
        return val;
    }

    void print_score() const {
        std::cout << "Final score: " << score << std::endl;
    }
};