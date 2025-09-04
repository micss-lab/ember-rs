#!/bin/bash

SESSION="smart-home-distributed"

if [ -n "$1" ]; then
    case "$1" in 
      "door")
        nix develop --impure --command \
        env \
            CASE_STUDY_SSID="hofterdoele-internet-iot" \
            CASE_STUDY_AP_PASSWORD="notthesamepasswordasmymainwifi1234;)" \
        cargo run-esp --release --bin case-study-smart-home-door 
        ;;

      "plant") 
        nix develop --impure --command \
        env \
            CASE_STUDY_SSID="hofterdoele-internet-iot" \
            CASE_STUDY_AP_PASSWORD="notthesamepasswordasmymainwifi1234;)" \
        cargo run-esp --release --bin case-study-smart-home-plant 
        ;;

      "control") 
        nix develop --impure --command \
        env \
            CASE_STUDY_SSID="hofterdoele-internet-iot" \
            CASE_STUDY_AP_PASSWORD="notthesamepasswordasmymainwifi1234;)" \
        cargo run-esp --release --bin case-study-smart-home-control
        ;;

      *) 
        echo -n "Unknown launch argument"
        echo "$1"
        exit 1
        ;; 
    esac
    read -p "$*"
fi

if [ -n "$TMUX" ] && [ -z "$SPAWNED_FROM_SCRIPT" ]; then
    # Launch script in a new Alacritty, mark as spawned
    SPAWNED_FROM_SCRIPT=1 alacritty --working-directory "$PWD" --command bash -c "$0" --hold
    exit
fi

# Otherwise, we're not in tmux: set up tmux panes as needed in this terminal
tmux new-session   -d -s $SESSION   "$0 door"
tmux split-window  -h -t $SESSION:1 "$0 plant"
tmux split-window  -v -t $SESSION:1 "$0 control"
tmux select-layout    -t $SESSION:1 tiled
tmux attach           -t $SESSION:1
