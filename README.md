# Turtle swarm 
This is a project which will enable mass coordination of turtles from Computercraft in Minecraft.
It will handle movement and control at a higher level, such as giving coordinate targets.
Additionally, turtles will have a unified memory and database with the manager program.

This higher capability along with a multi-turtle approach to the algorithms should allow for
much faster coordination and processing with different tasks like mining and building.
Potentially a swarm mode in the future to build up the turtles to some carrying capacity automatically.

Turtles will be controlled via a command queue in Lua. This has two modes, execute and query.
Execute will take a command to do some sort of action, like move, and complete it. Returns a success or failure.
Query on the other hand will take some question for the turtle which cannot fail (unless something is VERY wrong) and respond to the server with it.
The rust program will hold a local cache of coordinates and rotation. It will manage the dead-reckoning system.
Dead-reckoning will be used when there is no in-game GPS setup for the turtles.

There will be a task queue. Any idle turtle will take a task from the queue and start executing it. Whole jobs are handled
via task decomposition. Once a task is complete, it returns to IDLE.

### Task examples
- MoveTo(x, y, z)
- Dig(x, y, z)
- Craft(items[9])
- Place(x, y, z, block)
- Drop(x, y, z, item)
- Suck(x, y, z)
- Refuel()

## Problems:
Fresh turtle setup is a little rough, needs help fuelling and such

## Goals:
1. [x] Setup server communications, registering/dropping clients
2. [x] Setup database memory system
3. [x] Create virtual turtle initializer and destroyer
4. [x] Create dead-reckoning system for turtles
5. [x] Create virtual turtle remote control for testing
6. [x] Create pathfinding endpoint for clients to query
7. [x] Implement multi-turtle pathfinding at the same time
8. [ ] Implement state machine to handle complex logic
    - Building: Builds some sort of 3d model or schematic
    - Mining: Clears the entire area selected
    - Strip Mining: Mines underground efficiently for materials
    - Swarm: Combines strip mining with a reproduction state
    - Idle: does nothing
    - RC: Turtle is being remotely controlled

## Project structure
`./server/` contains the API server which manages all the turtles.  
`./turtle/` contains the turtle client's Lua functions and runtimes.  
`./client/` contains the user client for control and display.  
