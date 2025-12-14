# ECE1724F-Project-Final-Report

# Simple 2D Game Engine

### **Team Information:**

Yijun Chen, 1003045518, liloliver.chen@mail.utoronto.ca

Bart Cui, 1011827908, bart.cui@mail.utoronto.ca

---

## **1. Motivation**

Our motivation for picking this project comes from both personal interest and curiosity about how modern game engines are built. We heard that more gaming companies are starting to adapt existing large-scale game engines such as Unity and Unreal. These engines are powerful, but also very complex with hidden tools and APIs. Building even a small engine of our own gives us the chance to gain a deeper understanding of how engines operate and how these core components such as scene management, ECS design, and real-time rendering cooperate with each other.

Also, since Rust’s emphasis is on memory safety and concurrency without a garbage collector makes it particularly appealing for systems-level development. Having used C++ in past projects, we’re hoping to explore how Rust’s ownership model can improve both developer productivity and runtime safety in a game engine context. Given the scope of a course project, we plan to focus on implementing a minimal prototype featuring a basic rendering pipeline, level loading, and a simplified ECS framework.

Based on our research on game engine design and Bevy, we found that Bevy is good at real-time, parallel systems but is not directly aimed at turn-based, grid-based games. In such games, reproducibility is crucial and identical input sequences should always lead to the same outcomes. Bevy does not guarantee this behaviour by default. To address this, we will try to implement a custom turn scheduler that ensures consistent and reproducible state transitions. Although the new Bevy 0.17 recently introduced initial tilemap rendering support, it still lacks native grid utilities such as coordinate-to-world mapping, occupancy management, and pathfinding. That is why we try to build the engine to fill this space by adding a grid-aware foundation and reusable utilities designed for the needs of turn-based puzzle design, and hopefully can represent a small but meaningful contribution to the Rust game development ecosystem.

---

## **2. Objective**

The objective of this project is to design and implement a compact 2D game engine in Rust, built on top of Bevy for turn-based and grid-based puzzle games. The engine should use Bevy’s ECS architecture, scheduling system and plugin model. It aims to provide a data-driven structure where levels can be authored in external JSON files, loaded at runtime, and played through a consistent turn cycle using Bevy’s ECS and scheduling system.

The system is designed to handle core gameplay components such as player actions, simple AI behaviour, collisions, traps, doors, and goal detection, all operating on a tile-based grid with occupancy tracking and world-grid coordinate mapping. To support a complete gameplay loop, the engine includes scene management for navigating between the main menu, gameplay, pause overlay, level completion screen, and final game over state. A turn counter HUD, pause menu, and level-progression system enable users to experience multiple stages in sequence.

Overall, the project aims to provide a functional foundation for building small deterministic puzzle games, offering clear state transitions, reproducible turn logic, and an easy-to-extend architecture for future gameplay features.

## **3. Features**

Here is a list of features form our game engine which we will be discussed in details:

- [Grid System and Coordinate Mapping](#41-grid-system-and-coordinate-mapping)
- [Level Loading and Validation](#42-level-loading-and-validation)
- [Scene Management](#43-scene-management)
- [Level Progression System](#44-level-progression-system)
- [Save and Load System](#45-save-and-load-system)

### 4.1 Grid System and Coordinate Mapping

A complete grid abstraction layer was implemented to connect logical game coordinates to on-screen positions. Any game implemented on this engine can query the grid directly and does not have to compute transforms manually. This layer includes:

- **GridCoord** structure that represents a tile position using (x, y) coordinates and provides utility constructors and arithmetic helpers for directional movement.

- **GridTransform** that converts grid coordinates into world-space positions via uniform tile size and ensures that all sprites placed at **GridCoord** appear consistently aligned on screen. For example:

  - to_world(coord) → Vec3
  - to_grid(world_position) → GridCoord

- **OccupancyIndex**, it tracks which entities occupy each grid tile and supports multiple layers and tile queries. It is used during turn resolution to detect collisions, blocking, and goal triggers. Automatically rebuilt each turn to keep ECS state consistent.

### 4.2 Level Loading and Validation

A flexible level loader was implemented to allow developers to define levels using simple JSON files which include all entities' spawn positions. Levels can be authored entirely in data files and JSON deserialization with error checking to verify:

- Map bounds
- Valid tile types
- Duplicate entries
- Automatic creation of Bevy entities for each object type

### 4.3 Scene Management

A complete scene management system was implemented using Bevy’s States. The game behaves predictably and transitions cleanly between all major screens.

States we implemented:

- Menu: Main menu UI, start/load game buttons
- InGame: Active gameplay state
- GameOver: Win screen after all levels completed
- Pause Overlay: A UI overlay within InGame

Features:

- Automatic cleanup when switching scenes (despawns entities tied to a scene)
- Reconstruction of all game objects when entering a level
- Safe separation between "engine running" and "paused" states

### 4.4 Level Progression System

The engine now supports multi-level puzzle games and clean progression loops. A level progression structure was added to track the current level index (0 → N-1). It automatically advances upon finishing a level. When the player reaches a goal, a level complete window pops up with options:

- Next Level
- Return to Menu

If the player has completed all available levels:

- Displays a dedicated Game Over window
- Provides a button to return to the menu

### 4.5 Save and Load System

A simple save and load system was implemented to provide basic game persistence without requiring full world serialization. The engine introduces a SaveSlot resource that records whether a save exists and which level index the player last reached. Saving is triggered from the pause window via the “Save Game” option, which simply stores the current level index. Loading is available from the main menu through the “Load Game” option, which restores the saved level and starts it from the beginning. This mechanism allows players to leave the game and later continue their progression, offering a user-friendly solution.

### 4.6 Deterministic Turn Scheduler

A fully deterministic turn scheduler was implemented to guarantee reproducible gameplay outcomes across runs, which is essential for debugging, replay, and fair turn-based logic. The engine uses a fixed, explicitly ordered turn pipeline executed inside Bevy’s `Update` schedule:

**Input → AI Planning → Resolve → Commit → Cleanup**

- Input: Player input is collected and translated into high-level _intents_ (e.g., move, wait, interact) without mutating world state.
- AI Planning: AI-controlled entities observe the current state and produce their own intents deterministically.
- Resolve: All intents are evaluated together, and conflicts are resolved using explicit tie-breaking rules (e.g., movement collisions, priority ordering).
- Commit: State mutations are applied in a single-threaded, strictly ordered step to ensure determinism.
- Cleanup: Temporary intent data and per-turn caches are cleared, preparing the world for the next turn.

To eliminate non-deterministic behavior:

- All gameplay-affecting systems run in a fixed order.
- A seeded random number generator is used for any stochastic behavior.
- Parallel execution is avoided during the commit phase.

### 4.7 ECS for Game Objects

All game entities are modeled using Bevy’s Entity-Component-System (ECS) architecture, which provides clear separation between data and behavior and enables flexible composition of gameplay objects.

Each object is defined as a combination of small, reusable components, such as:

- `Position(GridCoord)`
- `Player`
- `Blocking`
- `Goal`
- `Trap`
- `Actor`
- `AI`

For example:

- A wall is composed of `Position + Blocking`
- An exit tile is composed of `Position + Goal`
- An enemy ghost is composed of `Position + Actor + AI + Blocking`

Systems operate over queries of components and are aligned with the deterministic turn pipeline. Importantly, systems **do not mutate persistent world state directly** during planning or resolution phases; all authoritative state changes occur only during the commit phase. This constraint greatly simplifies reasoning about game logic and helps maintain determinism across turns.

![Class diagram](./classes.JPG)

### 4.8 Pathfinding Algorithm

The engine uses **A\*** as its default pathfinding algorithm for AI-controlled entities. A\* was selected because it guarantees optimal paths like Dijkstra’s algorithm while exploring significantly fewer nodes when guided by an admissible heuristic.

Key characteristics of the implementation:

- Operates on the grid coordinate system using 4-connected movement.
- Uses Manhattan distance (|dx| + |dy|) as the heuristic, which is admissible and consistent for grid-based movement.
- Fully deterministic: identical inputs always produce identical paths.

Passability rules and movement costs are defined through a pluggable policy interface (implemented as a Rust trait). This allows different behaviors without modifying the core solver, such as:

- Player vs. enemy movement rules
- Doors, keys, or locked tiles
- Terrain-based movement costs

This modular design keeps pathfinding logic reusable, extensible, and easy to test in isolation.

### 4.9 Replay System for Deterministic Debugging

To aid debugging and validation of the deterministic state machine, a lightweight **replay system** was implemented. Instead of recording full world snapshots, the engine logs:

- The initial RNG seed
- The sequence of player input intents per turn

During replay mode, the engine reinitializes the world with the same seed and replays the recorded input stream through the deterministic turn scheduler. Because all systems are deterministic by construction, the replay reproduces the exact same sequence of states and outcomes.

This system enables:

- Step-by-step debugging of complex turn interactions
- Verification of bug fixes by comparing before/after replays
- “Golden tests” that assert identical end states for known input sequences

The replay mechanism proved especially valuable for diagnosing subtle ordering bugs and validating that refactors did not introduce non-deterministic behavior.

---

## 4. Developer’s Guide

This section explains how a user or developer can use the main features provided by the project deliverable. The engine is designed so that simple games require minimal setup, while more advanced projects can extend core systems without modifying existing logic.

### 4.1 Game Control and Key Binding

The default control scheme uses keyboard input mapped to grid-based movement:

- **W**: Move up
- **S**: Move down
- **A**: Move left
- **D**: Move right

Player input is captured each turn and translated into intent components rather than directly mutating game state. This allows input handling to remain deterministic and easily replayable.

Developers can:

- Remap keys by modifying the input system.
- Add new actions (e.g., interact, wait, special ability) by introducing new intent types and handling them in the resolve/commit phases of the turn pipeline.

---

### 4.2 Creating and Loading Levels

For basic usage, developers only need to create **JSON level files** and place them under:

```json
assets/levels/
```

Each level file defines:

- Map dimensions
- Tile layout
- Initial spawn positions for entities

Example level template files are given for developers to build on top of them:

```json
{
  "width": 10,
  "height": 8,
  "player_start": { "x": 1, "y": 1 },
  "walls": [
    { "x": 0, "y": 0 },
    { "x": 1, "y": 0 },
    { "x": 2, "y": 0 },
    { "x": 3, "y": 0 },
    { "x": 2, "y": 1 },
    { "x": -1, "y": 0 },
    { "x": -2, "y": 0 }
  ],
  "goals": [
    { "x": 8, "y": 6 }
  ],
  "enemies": [
    { "x": 8, "y": 5, "kind": "ghost" }
  ]
}
```

After creating a new level, add it to the LevelProgress in scenes/mod.rs

```rust
impl Default for LevelProgress {
    fn default() -> Self {
        Self {
            level_paths: vec![
                "assets/levels/level1.json".to_string(),
                "assets/levels/level2.json".to_string(),
                "assets/levels/level3.json".to_string(),
                // add more here later
            ],
            current: 0,
        }
    }
}
```

The engine automatically:

- Parses the JSON file
- Validates level correctness
- Spawns the appropriate Bevy entities with predefined components

### 4.3 Save and Load Usage

The save and load system is fully integrated into the engine workflow and requires no additional setup.

### 4.4 Deterministic Turn Pipeline Integration

All gameplay logic runs through a fixed deterministic pipeline:

Input → AI Planning → Resolve → Commit → Cleanup

For basic games, developers do not need to interact with this pipeline directly.

For advanced usage, developers can:

- Insert custom systems into a specific pipeline stage.
- Add new intent types that participate in conflict resolution.
- Introduce new rules during the resolve phase (e.g., priority movement, special interactions).

### 4.5 Extending Game Objects with ECS

Game objects are defined by combining components rather than creating rigid class hierarchies.

To add a new object type, a developer typically:

1. Defines one or more new components.
2. Spawns entities using those components in the level loader or setup system.
3. Adds systems that operate on those components within the turn pipeline.

### 4.6 Using Pathfinding for AI

AI-controlled entities automatically use the built-in A\* pathfinding system.

Developers can:

- Enable pathfinding by attaching AI-related components to an entity.
- Customize movement rules or costs by implementing a new pathfinding policy.
- Swap heuristics or constraints without modifying the core solver.

### 4.7 Replay System for Debugging and Testing

The replay system is primarily intended for developers.

To use it:

- Run the game normally while input and RNG seed are logged.
- Replay the session by feeding the recorded inputs back into the engine.

This allows developers to:

- Reproduce bugs exactly.
- Step through turns deterministically.
- Verify that changes to systems do not alter known outcomes.

---

## **5. Reproducibility Guide**

This project is a Rust + Bevy application. The steps below describe exactly how to set up the runtime environment and build/run the project on **Ubuntu Linux** and **macOS Sonoma**.

> Assumption: the instructor has terminal access and can install packages on the machine.

### 5.1 Get the Source Code

Clone the repository and enter it:

```bash
git clone <REPO_URL>
cd <REPO_DIR>
```

### 5.2 Build the Project

From the repo root:

```bash
cargo build
```

### 5.3 Run the Project

From the repo root:

```bash
cargo run
```

Run in release mode

```bash
cargo run --release
```

Once running:

Use W/A/S/D to move (grid-based movement).
Use the in-game menu options for Save / Load (if included in your build).
Levels are loaded from assets/levels/.

### 5.4 Verify Level Loading (JSON Levels)

To use the level loading feature, ensure JSON level files exist in:

```json
assets/levels/
```

---

## **6. Contributions**

Oliver focused on the design and implementation of the core engine loop and the deterministic turn scheduler. Each game turn is executed as a fixed, explicitly ordered pipeline (Input → AI Planning → Resolve → Commit → Cleanup), ensuring reproducible state transitions. Oliver implemented the collision, win, and lose rules, including stochastic deterministic logic using a seeded random number generator, as well as a replay system for debugging deterministic behavior.

This work includes defining clear conditions for game outcomes, such as player–enemy collisions and level completion when the player reaches an exit. A deterministic conflict resolution mechanism was implemented for cases where multiple actors target the same grid tile in a single turn. Oliver also implemented a grid-based A\* pathfinding algorithm that allows enemies to plan shortest paths around obstacles, using injected passability policies and a Manhattan-distance heuristic. The implementation enforces fixed neighbor ordering and stable priority-queue tie-breakers to maintain determinism.

Additional contributions include support for movement constraints involving doors, keys, traps, and walls via a pluggable policy interface, as well as the creation of golden replay tests (same RNG seed and input sequence ⇒ identical end states) to validate end-to-end determinism of the turn pipeline. All of these features were completed and integrated by the beginning of week 4.

Bart focused on the data and presentation layer that connected the logic to what players actually saw on screen. The main game objects were designed using small and reusable components such as Position, Actor, Blocking, Goal, Trap, and Door. Bart also built grid utilities that mapped between grid coordinates and on-screen positions to track which cells were occupied, and helped with queries such as finding neighbours or reachable areas. A simple level loader was implemented that read levels from text or JSON files and verified that each level was valid. Input mapping was also handled here, turning key presses defined in a config file into movement commands. Finally, the basic 2D rendering was set up so that the grid, characters, and a small status display, such as a turn counter, restart button, and seed number, were visible. For scene management, the engine used states for the Main Menu, the In Game session, and simpler overlays for temporary screens such as Pause and Game Over. When the player returned to the main menu or restarted a level, the previous game entities were cleared and the state reset.

After integrating the components, the team shipped a small, polished chasing demo that proved the engine worked end-to-end. Multiple rounds of testing were conducted to ensure the movement rules functioned smoothly within the grid and rendering system. By week 5, the team worked on the simple game demo, showcased the engine’s features, and provided the final proof that the framework worked as intended.


---

## **Lessons learned and concluding remarks**

One of the most important lessons from this project was the value of a deterministic turn scheduler in managing complex game logic. By enforcing a fixed, explicitly ordered turn pipeline (Input → AI Planning → Resolve → Commit → Cleanup), we significantly reduced the difficulty of debugging gameplay behavior. Determinism made it possible to reason about the system one turn at a time, ensured that identical inputs always produced identical outcomes, and enabled powerful tooling such as replay-based debugging and golden tests. This approach highlighted how careful system ordering and clear phase boundaries can transform an otherwise fragile, state-heavy game loop into a predictable and testable state machine.


Another key takeaway was how Rust’s ownership and borrowing model helped prevent entire classes of runtime errors before the program ever ran. Constraints enforced by the compiler—such as exclusive mutable access, explicit lifetimes, and clear data ownership—initially slowed development but ultimately led to safer and more maintainable code. Many potential bugs common in game engines, including accidental shared mutation, use-after-free errors, and hidden data races, were caught at compile time. Combined with ECS patterns, Rust’s type system encouraged designing systems with explicit data dependencies, which aligned naturally with the deterministic turn scheduler and reduced runtime failures.
