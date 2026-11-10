# Pomodoro TUI

A minimal terminal Pomodoro timer built with [Ratatui](https://ratatui.rs/) and Crossterm.

```sh
cargo run
```

Press **Space** to start or pause, **r** to reset the current phase, **n** to skip to the next phase, or **q** to quit. Focus sessions last 25 minutes, short breaks 5 minutes, and every fourth focus session is followed by a 15-minute break.
