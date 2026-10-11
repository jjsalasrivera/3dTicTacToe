# Tres en Raya 3D

![Rust](https://img.shields.io/badge/Rust-1.95%2B-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-0.19-blue)
![License](https://img.shields.io/badge/license-GPL--3.0-blue)

Juego de **tres en raya tridimensional** (rejilla **3×3×3 con gravedad**) sobre una
mesa de madera con nueve varillas, escrito en **Rust** con el motor **Bevy**.

Las fichas caen por gravedad y se apilan en las columnas. Gana quien alinee **tres
fichas** en cualquiera de las **49 líneas** del cubo (ejes, diagonales de cara y
diagonales espaciales).

---

## 🎮 Cómo se juega

- El tablero es un cubo de **3×3×3 = 27 celdas**.
- El eje **Z es vertical** (gravedad): al jugar en una columna `(x, y)`, la ficha cae
  al **primer nivel libre** (0, 1 o 2) y se apila sobre las anteriores.
- Se gana alineando **3 fichas** en cualquiera de las **49 líneas** del cubo
  (ejes, diagonales de cara y diagonales espaciales como `(0,0,0) – (1,1,1) – (2,2,2)`).
- La **primera jugada no puede ir al centro** de la base `(1,1,0)`.
- En el menú eliges si empiezas **tú** o la **computadora**, y la **dificultad**.

### Controles

| Acción | Control |
|---|---|
| Rotar el tablero | Arrastrar con el **botón izquierdo** |
| Acercar / alejar | **Rueda del ratón** |
| Colocar una ficha | **Clic izquierdo** en la columna bajo el cursor |

La columna bajo el cursor se resalta cuando es jugable.

### Fichas

| Jugador | Ficha | Color |
|---|---|---|
| Tú | **X** (dos barras cruzadas en 3D) | Rojo |
| Computadora | **O** (anillo circular en 3D) | Azul |

---

## 🚀 Ejecutar

Requisitos: un toolchain de Rust reciente. **Bevy 0.19 exige rustc ≥ 1.95**:

```bash
rustup update stable
```

Compilar y jugar (se recomienda modo *release* por rendimiento):

```bash
cargo run --release
```

Durante el desarrollo:

```bash
cargo run
```

### Tests

```bash
cargo test
```

Cubren: las 49 líneas, la gravedad y el límite de altura por columna, la
restricción del centro, el deshacer jugadas (`pop`) y el comportamiento básico de
la IA (jugada legal y bloqueo de amenazas).

---

## 🧠 La IA

- **Negamax con poda alfa-beta**.
- **Tabla de transposición** con *hashing* **Zobrist**.
- **Ordenación de jugadas** (centro → esquinas → bordes) y **profundización
  iterativa** con presupuesto de nodos (algoritmo *anytime*).
- Evaluación heurística sobre las 49 líneas (líneas muertas y amenazas de 2).
- Dificultades: **Fácil** (con algo de azar), **Medio** y **Difícil**.

El modelo del tablero usa **bitboards de 27 bits** (`bit = columna * 3 + nivel`), de
modo que comprobar una victoria es una operación de bits contra 49 máscaras.

---

## 🗂️ Estructura

| Archivo | Responsabilidad |
|---|---|
| `src/main.rs` | Arranque, ventana, estados y registro de plugins |
| `src/config.rs` | Constantes y conversión entre coordenadas del juego y del mundo |
| `src/board.rs` | Tablero, reglas, bitboards y las 49 líneas ganadoras |
| `src/game.rs` | Recurso de partida: turno, fase, dificultad y resultado |
| `src/ai.rs` | Negamax + alfa-beta, transposición Zobrist y profundización iterativa |
| `src/render.rs` | Mesa de madera, varillas, luces y línea ganadora |
| `src/pieces.rs` | Meshes/materiales de las fichas y animación de caída |
| `src/camera.rs` | Cámara orbital ([`bevy_panorbit_camera`](https://github.com/Plonq/bevy_panorbit_camera)) |
| `src/input.rs` | Raycast del ratón, resaltado de columna y colocación |
| `src/ui.rs` | Menú, HUD y pantalla de fin de partida |
| `src/state.rs` | Estados (`Menu`, `Playing`, `GameOver`) |

---

## 📄 Licencia

Este proyecto se distribuye bajo la **GNU General Public License v3.0**. Consulta
el archivo [`LICENSE`](LICENSE) para más detalles.
