# 3D Tic-Tac-Toe

![Rust](https://img.shields.io/badge/Rust-1.95%2B-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-0.19-blue)
![License](https://img.shields.io/badge/license-GPL--3.0-blue)

A **three-dimensional tic-tac-toe** game (a **3×3×3 grid with gravity**) on a wooden
table with nine rods, written in **Rust** with the **Bevy** engine.

Pieces fall under gravity and stack inside the columns. You win by lining up
**three pieces** along any of the **49 lines** of the cube (axes, face diagonals and
space diagonals).

---

## 🎮 How to play

- The board is a **3×3×3 = 27 cell** cube.
- The **Z axis is vertical** (gravity): when you play a column `(x, y)`, the piece
  falls to the **lowest free level** (0, 1 or 2) and stacks on top of the others.
- You win by lining up **3 pieces** along any of the **49 lines** of the cube
  (axes, face diagonals and space diagonals such as `(0,0,0) – (1,1,1) – (2,2,2)`).
- The **first move cannot be the center** of the base `(1,1,0)`.
- In the menu you choose whether **you** or the **computer** go first, and the
  **difficulty**.

### Controls

| Action | Control |
|---|---|
| Rotate the board | **Left-drag** |
| Zoom in / out | **Scroll wheel** |
| Place a piece | **Left-click** the column under the cursor |

The column under the cursor is highlighted when it is playable.

### Pieces

| Player | Piece | Color |
|---|---|---|
| You | **X** (two crossed 3D bars) | Red |
| Computer | **O** (3D ring) | Blue |

---

## 🚀 Running

Requirements: a recent Rust toolchain. **Bevy 0.19 requires rustc ≥ 1.95**:

```bash
rustup update stable
```

Build and play (*release* mode is recommended for performance):

```bash
cargo run --release
```

During development:

```bash
cargo run
```

### Tests

```bash
cargo test
```

They cover: the 49 winning lines, gravity and the per-column height limit, the
center restriction, undoing moves (`pop`) and the basic AI behaviour (legal move
and threat blocking).

---

## 🧠 The AI

- **Negamax with alpha-beta pruning**.
- **Transposition table** with **Zobrist hashing**.
- **Move ordering** (center → corners → edges) and **iterative deepening** with a
  node budget (an *anytime* algorithm).
- Heuristic evaluation over the 49 lines (dead lines and 2-piece threats).
- Difficulties: **Easy** (partly random), **Medium** and **Hard**.

The board model uses **27-bit bitboards** (`bit = column * 3 + level`), so checking
for a win is a bitwise operation against 49 precomputed masks.

---

## 🗂️ Project structure

| File | Responsibility |
|---|---|
| `src/main.rs` | Startup, window, states and plugin registration |
| `src/config.rs` | Constants and conversion between game and world coordinates |
| `src/board.rs` | Board, rules, bitboards and the 49 winning lines |
| `src/game.rs` | Match resource: turn, phase, difficulty and outcome |
| `src/ai.rs` | Negamax + alpha-beta, Zobrist transposition and iterative deepening |
| `src/render.rs` | Wooden table, rods, lights and the winning line |
| `src/pieces.rs` | Piece meshes/materials and fall animation |
| `src/camera.rs` | Orbital camera ([`bevy_panorbit_camera`](https://github.com/Plonq/bevy_panorbit_camera)) |
| `src/input.rs` | Mouse raycast, column highlight and placement |
| `src/ui.rs` | Menu, HUD and game-over screen |
| `src/state.rs` | States (`Menu`, `Playing`, `GameOver`) |

---

## 📄 License

This project is distributed under the **GNU General Public License v3.0**. See the
[`LICENSE`](LICENSE) file for details.
