# sudoku
A sudoku game written in Rust using ggez.

![](screenshot.png)

The sudoku engine can support n-sized sudokus but the game only uses standard 9x9 grids. You can still modify the source code to play with bigger (or smaller) sudoku grids and everything will still work.

Certain tiles are colored green, and are a step in the right direction for the solution (no other values can be placed in their spot, or the value cant be place anywhere else in the row, column, or quadrant).

Uncertain tiles are colored black, opposite of certain tiles, contain values that can be placed there, but we can replace with others and still have a solvable table.

The algorithm to determine if a tile is certain or not can be found in `update_correct` of `main.rs`.

## Running

Just type
```
cargo run --release
```

## Controls
* `1-9`: Select the value to place
* `Tab`: Automatically place a new value
* `LeftClick`: Place a tile with the selected value
* `RightClick`: Remove a tile (if it's uncertain)

When the grid is full the game automatically resets.
