# Turtle swarm 
This is a project which will enable mass coordination of turtles from Computercraft in Minecraft.
It will handle movement and control at a higher level, such as giving coordinates targets.
Additionally, turtles will have a unified memory and database with the manager program.

This higher capability along with a multi-turtle approach to the algorithms should allow for
much faster coordination and processing with different tasks like mining and building.
Potentially a swarm mode in the future to build up the turtles to some carrying capacity automatically.

Turtles will be controlled via an interface layer in Lua. This interface will be referenced as 'VirtualTurtle'
and will hold a local cache of coordinates and rotation. It will manage the dead-reckoning system.
Dead-reckoning will be used when there is no in-game GPS setup for the turtles.

## Goals:
1. [ ] Setup server communications, registering/dropping clients
2. [ ] Setup database memory system
3. [ ] Create virtual turtle initializer and destroyer
4. [ ] Create dead-reckoning system for turtles
5. [ ] Create virtual turtle remote control for testing
6. [ ] Create pathfinding endpoint for clients to query
7. [ ] Implement multi-turtle pathfinding at the same time
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
