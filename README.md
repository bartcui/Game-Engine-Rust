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

## **4. Developer's Guide**

Game control/key binding
WSAD for directions

For basic use, devs just write JSON level files and drop them in assets/levels/.
For more advanced games, they can extend the component set and plug additional systems into the same deterministic turn pipeline.

---

## **5. Reproducibility Guide**

---

## **6. Contributions**

Oliver will focus on the core engine loop and the turn scheduler. Each turn will be structured as a fixed pipeline (Input → AI Plan→ Resolve → Commit → Cleanup). Oliver will also implement collision/win/lose rules, tie-breakers with RNG and replay for debugging, and a basic pathfinding algorithm (A\*) that allows enemies to plan shortest routes around obstacles that use the injected passability and Manhattan heuristic.

This includes implementing the conditions for collisions and win/lose states, such as what happens when the player reaches an exit or when an enemy catches the player, a deterministic conflict resolution when multiple actors target the same tile. Furthermore, movement that respects to objects like doors/keys, traps, and walls via the pluggable policy; fixed neighbour ordering and stable priority-queue tie-breakers in A\*; and golden-replay tests (same seed + inputs ⇒ identical outcomes) to verify the end-to-end determinism of the turn pipeline. By the start of week 4, all these features should be developed and ready for integration.

Bart will focus on the data and presentation layer that connects the logic to what players actually see on screen. The main game objects will be designed using small and reusable components such as Position, Actor, Blocking, Goal, Trap and Door. Bart will also build grid utilities that map between grid coordinates and on-screen positions to track which cells are occupied, and help with queries such as finding neighbours or reachable areas. A simple level loader will be implemented that reads levels from text or JSON files and verifies that each level is valid. Input mapping will also be handled here, turning key presses defined in a config file into movement commands. Finally, the basic 2D rendering will be set up so that the grid, characters, and a small status display, such as a turn counter, restart button, and seed number, are visible. For scene management, the engine will use states for the Main Menu, the In Game session, and simpler overlays for temporary screens such as Pause and Game Over. When the player returns to the main menu or restarts a level, the previous game entities will be cleared and the state resets.

After integrating our components, we will ship a small, polished chasing demo that proves the engine works end-to-end. Multiple rounds of testing are necessary to ensure the movement rules function smoothly within the grid and rendering system. By week 5, we will work on the simple game demo, showcase the engine’s features and the final proof that the framework works as intended.

---

## **Lessons learned and concluding remarks**
