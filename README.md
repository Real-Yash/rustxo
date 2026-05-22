# rust-xo

A Rust desktop tic tac toe game for Linux/Arch.

## Features

- Local two-player mode.
- Computer mode with non-AI game algorithms:
  - Classic 3x3 uses minimax with alpha-beta pruning.
  - Super mode uses tactical heuristic scoring for the larger search space.
- Classic 3x3 tic tac toe.
- Super tic tac toe: each main square contains its own 3x3 board. The cell a player chooses sends the next player to the matching mini-board. If that board is already won or full, the next player can move anywhere.
- Animated marks, active-board pulse, moving background particles, and win highlights.


## Why?

So i have been learning Rust lately, and if you know me, you must be knew my love for arch, so i thought why not build an classic tic tac toe game for arch. so i built this an XO game written in rust.


## Run

```sh
cargo run
```
