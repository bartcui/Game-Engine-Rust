# ECE1724F-Project-Project-Proposal
# Simple Game Engine
Bart Cui - 1011827908
Yijun(Oliver) Chen - 1003045518
## Motivation

Our motivation for picking this project comes from both personal interest and curiosity about how modern game engines are built. We heard that more gaming companies are starting to adapt existing large-scale game engines such as Unity and Unreal. These engines are powerful, but also very complex with hidden tools and APIs. Building even a small engine of our own gives us the chance to gain a deeper understanding of how engines operate and how these core components such as scene management, ECS design, and real-time rendering cooperate with each other.

Also, since Rust’s emphasis is on memory safety and concurrency without a garbage collector makes it particularly appealing for systems-level development. Having used C++ in past projects, we’re hoping to explore how Rust’s ownership model can improve both developer productivity and runtime safety in a game engine context. Given the scope of a course project, we plan to focus on implementing a minimal prototype featuring a basic rendering pipeline, level loading, and a simplified ECS framework. 

## Novelty

Since we are new to the Rust ecosystem, we are not fully aware of all the gaps that currently exist or which ones need the most attention. However, based on our research on game engine design and Bevy, we found that Bevy is good at real-time, parallel systems but is not directly aimed at turn-based, grid-based games. In such games, reproducibility is crucial and identical input sequences should always lead to the same outcomes. Bevy does not guarantee this behaviour by default. To address this, we will try to implement a custom turn scheduler that ensures consistent and reproducible state transitions.

Although the new Bevy 0.17 recently introduced initial tilemap rendering support, it still lacks native grid utilities such as coordinate-to-world mapping, occupancy management, and pathfinding. We will try to build the engine to fill this space by adding a grid-aware foundation and reusable utilities designed for the needs of turn-based puzzle design, and hopefully can represent a small but meaningful contribution to the Rust game development ecosystem.

## Objective

Our goal is to design a small 2D game engine in Rust, built on top of Bevy for turn-based and grid-based puzzle games. We will use Bevy’s ECS, scheduling, and plugin model, and extend it with a deterministic turn scheduler and a grid-native utility layer. The engine will allow game developers to load grid levels from data files and provide reproducible gameplay using seeded random numbers and replays. We will create a demo that showcases level loading, scene changing (Menu → Game In → Game Over), and simple gameplay with full player and opponent movements.

## Key Features

### Deterministic Turn Scheduler
Fixed pipeline (Input(config into intents) → AI Plan(state into AI intents)→ Resolve(conflict/ tie rules) → Commit → Cleanup), explicit system ordering, single-threaded commit step, seeded RNG. Deliverables: replay file (inputs + seed), golden tests that reproduce identical end states.

### Grid-Native Abstractions
Defines a GridCoord type and provides grid↔world mapping (tile size/origin), occupancy and layers (terrain/unit/item), blocking and collision flags, neighbour iterators (4- or 8-connected), and region queries (flood-fill).

### Map Loader
Supports level formats such as JSON, TOML, or simple ASCII layouts.  Validates bounds, layers, and blocking flags, and builds grid + occupancy indices on load.

### ECS for Game Objects
Every game object will be broken down into small components such as Position(GridCoord), Player, Blocking, Goal, Trap, and AI.  
For example: a wall = Position + Blocking; an exit = Position + Goal; a ghost = Position + Actor + AI + Blocking. Systems follow the turn pipeline and only mutate state during the commit phase.

### Scene Management
The engine organizes the game into scenes such as Main Menu, Game In, and Game Over. Each scene has its own setup and cleanup to help with smooth transitions and predictable behaviour.

### Pathfinding Algorithm
Uses A* as the default shortest-path solver. A* provides optimal routes like Dijkstra’s algorithm while exploring fewer nodes when guided by a heuristic. Passability and step costs are defined through a pluggable policy (a Rust trait), so different movers (player, ghost), door/key mechanics, or terrain costs can be swapped without altering the solver. The default heuristic is Manhattan distance (|dx| + |dy|), which is admissible and consistent on a 4-connected grid — fast to compute and ideal for deterministic turn logic.
![Class diagram](./classes.JPG)
## Tentative Plan

We aim to finish the project within the timeframe of five weeks and have enough time to polish and work on the report and demo in late November. First, we both need to get familiar with Bevy and game engine design concepts, focusing on ECS, plugins, states, assets, 2D rendering aspects. After the ramp-up, Oliver will focus on the core engine loop and the turn scheduler. Each turn will be structured as a fixed pipeline (Input → AI Plan→ Resolve → Commit → Cleanup). Oliver will also implement collision/win/lose rules, tie-breakers with RNG and replay for debugging, and a basic pathfinding algorithm (A*) that allows enemies to plan shortest routes around obstacles that use the injected passability and Manhattan heuristic. 

This includes implementing the conditions for collisions and win/lose states, such as what happens when the player reaches an exit or when an enemy catches the player, a deterministic conflict resolution when multiple actors target the same tile. Furthermore, movement that respects to objects like doors/keys, traps, and walls via the pluggable policy; fixed neighbour ordering and stable priority-queue tie-breakers in A*; and golden-replay tests (same seed + inputs ⇒ identical outcomes) to verify the end-to-end determinism of the turn pipeline. By the start of week 4, all these features should be developed and ready for integration.

Bart will focus on the data and presentation layer that connects the logic to what players actually see on screen. The main game objects will be designed using small and reusable components such as Position, Actor, Blocking, Goal, Trap and Door. Bart will also build grid utilities that map between grid coordinates and on-screen positions to track which cells are occupied, and help with queries such as finding neighbours or reachable areas. A simple level loader will be implemented that reads levels from text or JSON files and verifies that each level is valid. Input mapping will also be handled here, turning key presses defined in a config file into movement commands. Finally, the basic 2D rendering will be set up so that the grid, characters, and a small status display, such as a turn counter, restart button, and seed number, are visible. For scene management, the engine will use states for the Main Menu, the In Game session, and simpler overlays for temporary screens such as Pause and Game Over. When the player returns to the main menu or restarts a level, the previous game entities will be cleared and the state resets. 

After integrating our components, we will ship a small, polished chasing demo that proves the engine works end-to-end. Multiple rounds of testing are necessary to ensure the movement rules function smoothly within the grid and rendering system. By week 5, we will work on the simple game demo, showcase the engine’s features and the final proof that the framework works as intended. 


Game control/key binding
WSAD for directions
R -> restart









