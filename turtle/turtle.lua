local version = 2

-- This is a simple dumb terminal program for the turtle.
-- It is intended to be the part of the program which executes commands
-- given from the digital twin in rust.
local turtle_req_id = 0
local should_exit = false

local function query_server(socket, data)
    socket.send(textutils.serializeJSON({
        type = "Query",
        id = turtle_req_id + 1,
        data = data
    }))

    turtle_req_id = turtle_req_id + 1

    return socket.receive()
end

local action_table = {
    -- Movement
    ["Forward"] = function(args)
        local success, err = turtle.forward()
        return { success = success, reason = err }
    end,
    ["Back"] = function(args)
        local success, err = turtle.back()
        return { success = success, reason = err }
    end,
    ["Up"] = function(args)
        local success, err = turtle.up()
        return { success = success, reason = err }
    end,
    ["Down"] = function(args)
        local success, err = turtle.down()
        return { success = success, reason = err }
    end,
    ["TurnLeft"] = function(args)
        local success, err = turtle.turnLeft()
        return { success = success, reason = err }
    end,
    ["TurnRight"] = function(args)
        local success, err = turtle.turnRight()
        return { success = success, reason = err }
    end,

    -- World interactions
    ["Dig"] = function(args)
        local success, err = turtle.dig(args["side"])
        return { success = success, reason = err }
    end,
    ["DigUp"] = function(args)
        local success, err = turtle.digUp(args["side"])
        return { success = success, reason = err }
    end,
    ["DigDown"] = function(args)
        local success, err = turtle.digDown(args["side"])
        return { success = success, reason = err }
    end,
    ["Place"] = function(args)
        local success, err = turtle.place(args["text"])
        return { success = success, reason = err }
    end,
    ["PlaceUp"] = function(args)
        local success, err = turtle.placeUp(args["text"])
        return { success = success, reason = err }
    end,
    ["PlaceDown"] = function(args)
        local success, err = turtle.placeDown(args["text"])
        return { success = success, reason = err }
    end,
    ["Attack"] = function(args)
        local success, err = turtle.attack(args["side"])
        return { success = success, reason = err }
    end,
    ["AttackUp"] = function(args)
        local success, err = turtle.attackUp(args["side"])
        return { success = success, reason = err }
    end,
    ["AttackDown"] = function(args)
        local success, err = turtle.attackDown(args["side"])
        return { success = success, reason = err }
    end,

    -- Inventory
    ["Select"] = function(args)
        return { success = turtle.select(args["slot"]) }
    end,
    ["Drop"] = function(args)
        local success, err = turtle.drop(args["count"])
        return { success = success, reason = err }
    end,
    ["DropUp"] = function(args)
        local success, err = turtle.dropUp(args["count"])
        return { success = success, reason = err }
    end,
    ["DropDown"] = function(args)
        local success, err = turtle.dropDown(args["count"])
        return { success = success, reason = err }
    end,
    ["Suck"] = function(args)
        local success, err = turtle.suck(args["count"])
        return { success = success, reason = err }
    end,
    ["SuckUp"] = function(args)
        local success, err = turtle.suckUp(args["count"])
        return { success = success, reason = err }
    end,
    ["SuckDown"] = function(args)
        local success, err = turtle.suckDown(args["count"])
        return { success = success, reason = err }
    end,
    ["TransferTo"] = function(args)
        return { success = turtle.transferTo(args["slot"], args["count"]) }
    end,

    -- Fuel & slots
    ["Refuel"] = function(args)
        local success, err = turtle.refuel(args["count"])
        return { success = success, reason = err }
    end,
    ["EquipLeft"] = function(args)
        local success, err = turtle.equipLeft()
        return { success = success, reason = err }
    end,
    ["EquipRight"] = function(args)
        local success, err = turtle.equipRight()
        return { success = success, reason = err }
    end,

    -- Misc
    ["Craft"] = function(args)
        local success, err = turtle.craft(args["limit"])
        return { success = success, reason = err }
    end,
    ["Quit"] = function(args)
        should_exit = true
        return { success = true }
    end,

    -- Custom
    ["ChangeName"] = function(args)
        os.setComputerLabel(args["name"])
        return { success = true }
    end,
    ["Wait"] = function(args)
        sleep(0.6)
    end,
    ["StartGpsHost"] = function(args, socket, server_req_id)
        -- This starts a blocking job to host GPS
        local success = true

        function host()
            socket.send(textutils.serializeJSON({
                type = "Response",
                res_id = server_req_id,
                data = { success = true }
            }))

            success = shell.execute("gps", "host", tostring(args[1]), tostring(args[2]), tostring(args[3]))
        end

        function wait_for_stop_command()
            while true do
                local message, is_binary = socket.receive()
        
                if message then
                    local request = decapsulate_data(message)
                    
                    if request["type"] == "request" and request["data"]["action"] == "stop_gps_host" then
                        print("Received stop command.")

                        if not request["oneshot"] then
                            socket.send(encapsulate_data({
                                type = "response",
                                res_id = request["req_id"],
                                data = { success = true }
                            }))
                        end

                        return
                    elseif request["type"] == "request" then
                        print("Ignoring command, currently hosting GPS")

                        if not request["oneshot"] then
                            socket.send(encapsulate_data({
                                type = "response",
                                res_id = request["req_id"],
                                data = {
                                    success = false,
                                    reason = "GPS is currently running, cannot do anything else."
                                }
                            }))
                        end
                    end
                end
            end
        end

        parallel.waitForAny(host, wait_for_stop_command)
        print("GPS host successful: " .. tostring(success))
        return nil
    end,
    ["StopGpsHost"] = function(args)
        return { success = false, reason = "No GPS running." }
    end,
    ["UpdateLocation"] = function(args)
        local location_data = { x = args["x"], y = args["y"], z = args["z"], direction = args["direction"] }
        local file, err = fs.open("location.json", "w")

        if file then
            file.write(textutils.serializeJSON(location_data))
            file.close()
        end

        return { success = file ~= nil, reason = err }
    end,
}

local query_table = {
    -- World
    ["Detect"] = function(args)
        return { success = turtle.detect(), last_action = 0 }
    end,
    ["DetectUp"] = function(args)
        return { success = turtle.detectUp(), last_action = 0 }
    end,
    ["DetectDown"] = function(args)
        return { success = turtle.detectDown(), last_action = 0 }
    end,
    ["Inspect"] = function(args)
        local success, data = turtle.inspect()
        return { success = success, data = data, last_action = 0 }
    end,
    ["InspectUp"] = function(args)
        local success, data = turtle.inspectUp()
        return { success = success, data = data, last_action = 0 }
    end,
    ["InspectDown"] = function(args)
        local success, data = turtle.inspectDown()
        return { success = success, data = data, last_action = 0 }
    end,

    -- Inventory
    ["GetSelectedSlot"] = function(args)
        return { success = true, last_action = 0, data = turtle.getSelectedSlot() }
    end,
    ["GetItemCount"] = function(args)
        return { success = true, last_action = 0, data = turtle.getItemCount(args[1]) }
    end,
    ["GetItemSpace"] = function(args)
        return { success = true, last_action = 0, data = turtle.getItemSpace(args[1]) }
    end,
    ["GetItemDetail"] = function(args)
        return { success = true, last_action = 0, data = turtle.getItemDetail(args[1], args[2]) }
    end,
    ["Compare"] = function(args)
        return { success = true, last_action = 0, data = turtle.compare() }
    end,
    ["CompareUp"] = function(args)
        return { success = true, last_action = 0, data = turtle.compareUp() }
    end,
    ["CompareDown"] = function(args)
        return { success = true, last_action = 0, data = turtle.compareDown() }
    end,
    ["CompareTo"] = function(args)
        return { success = true, last_action = 0, data = turtle.compareTo(args[1]) }
    end,

    -- Fuel & slots
    ["GetFuelLevel"] = function(args)
        return { success = true, last_action = 0, data = turtle.getFuelLevel() }
    end,
    ["GetFuelLimit"] = function(args)
        return { success = true, last_action = 0, data = turtle.getFuelLimit() }
    end,
    ["GetEquippedLeft"] = function(args)
        return { success = true, last_action = 0, data = turtle.getEquippedLeft() }
    end,
    ["GetEquippedRight"] = function(args)
        return { success = true, last_action = 0, data = turtle.getEquippedRight() }
    end,

    -- Custom
    ["TurtleInit"] = function(args, socket)
        -- First get versions from server
        if args["version"] > version then
            print("Script is out of date!")
            sleep(4)

            if args["script"] then
                print("Update available! Installing now...")

                local newFile = fs.open("./startup.lua.new", "w")
                newFile.write(args["script"])
                newFile.close()

                fs.move("./startup.lua", "./startup.lua.old")
                fs.move("./startup.lua.new", "./startup.lua")
                os.reboot()
                sleep(1)
            end
                
            os.exit()
        else
            xpcall(function()
                fs.delete("./startup.lua.old")
            end, function()
            end )
        end

        -- Try location with GPS, then see if a saved state exists, then ask server to host GPS, all else fails then manual entry
        print(query_server(socket, { type = "Ping" }))
        local location_data
        local x, y, z = gps.locate()

        if x ~= nil then
            -- GPS works, move forward and get the next position to determine face
            local success = turtle.forward()
            while not success do
                sleep(0.5)
                success = turtle.forward()
            end

            local x2, y2, z2 = gps.locate()
            
            success = turtle.back()
            while not success do
                sleep(0.5)
                success = turtle.back()
            end

            local direction = "Unknown"
            if x2 > x then
                direction = "East"
            elseif x2 < x then
                direction = "West"
            elseif z2 > z then
                direction = "South"
            elseif z2 < z then
                direction = "North"
            end
            
            print("GPS location: " .. x .. ", " .. y .. ", " .. z .. " facing " .. direction)
            location_data = { x = x, y = y, z = z, direction = direction }

            local file = fs.open("location.json", "w")
            file.write(textutils.serializeJSON(location_data))
            file.close()
        elseif fs.exists("location.json") then
            local file = fs.open("location.json", "r")
            local content = file.readAll()
            file.close()
            location_data = textutils.unserializeJSON(content)
        else
            write("X=")
            local x = tonumber(read())
            write("Y=")
            local y = tonumber(read())
            write("Z=")
            local z = tonumber(read())
            write("Direction=")
            local direction = read()

            location_data = { x = x, y = y, z = z, direction = direction }

            local file = fs.open("location.json", "w")
            file.write(textutils.serializeJSON(location_data))
            file.close()
        end

        return { success = true, last_action = 0, data = { location_data.x, location_data.y, location_data.z, string.upper(location_data.direction)} }
    end
}

function handle_command(socket, data)
    -- This will parse the command, execute it, and send back the result to the server
    local msg_type = data["type"]
    local req_id = data["id"]

    if msg_type == "Query" then
        -- Server is asking the turtle something
        local command = data["data"]
        local query = query_table[command["type"]]

        if query then
            local result = query(command, socket, req_id)

            if result then
                socket.send(textutils.serializeJSON({
                    type = "Response",
                    id = req_id,
                    data = result
                }))
            end
        end
    elseif msg_type == "Procedure" then
        -- Server is telling the turtle to do something and expects a response
        local command_list = data["data"]
        local last_reason = nil
        local count = 0

        for k,command in pairs(command_list) do
            local action = action_table[command["action"]]

            if action then
                local result = action(command["args"], socket, req_id)

                if result and not result.success then
                    last_reason = result.reason
                    break
                end
            else
                last_reason = "Invalid action."
                break
            end

            count = count + 1
        end

        socket.send(textutils.serializeJSON({
            type = "Response",
            id = req_id,
            data = {
                success = (last_reason == nil),
                reason = last_reason,
                last_action = count
            }
        }))
    elseif msg_type == "Oneshot" then
        -- Server is telling the turtle to do something and doesn't care about what is has to say :(
        local command_list = data["data"]

        for k,v in pairs(command_list) do
            local action = action_table[command["action"]]

            if action then
                local result = action(command["args"], socket, req_id)

                if result and not result.success then
                    break
                end
            else
                break
            end
        end
    end
end

function main()
    -- Connect to websocket
    local SERVER_URL = "ws://localhost:8080"
    local ws = nil

    function runtime()
        print("Connecting to server at " .. SERVER_URL)
        ws = http.websocket(SERVER_URL)
    
        while not ws do
            error("Failed to connect to server")
            sleep(5) -- Wait before retrying
            print("Retrying connection to server at " .. SERVER_URL)
            ws = http.websocket(SERVER_URL)
        end
    
        -- Tell server we're a turtle
        ws.send("turtle")
        print("Connected to server at " .. SERVER_URL)
    
        -- Main runtime to listen to commands from the server
        while not should_exit do
            local message, is_binary = ws.receive()
    
            if message then
                local data = textutils.unserializeJSON(message)
                print("Received command: " .. message)
                handle_command(ws, data)
            else
                print("Connection to server lost. Attempting to reconnect...")
                ws.close()
                ws = nil
    
                while not ws do
                    sleep(5) -- Wait before retrying
                    print("Retrying connection to server at " .. SERVER_URL)
                    ws = http.websocket(SERVER_URL)
                end
    
                print("Reconnected to server at " .. SERVER_URL)
                ws.send("turtle")
            end
        end
    end

    local success, err = pcall(runtime)

    -- Print out the error problem
    while err ~= "Terminated" do
        print(err)
        sleep(2)
        success, err = pcall(runtime)
    end

    -- Cleanup main
    if ws then
        ws.close()
    end
end

main()