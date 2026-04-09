-- This is a simple dumb terminal program for the turtle.
-- It is intended to be the part of the program which executes commands
-- given from the digital twin in rust.
local turtle_req_id = 0

local action_table = {
    ["forward"] = function(args)
        local success, err = turtle.forward()
        return { success = success, error = err }
    end,
    ["back"] = function(args)
        local success, err = turtle.back()
        return { success = success, error = err }
    end,
    ["up"] = function(args)
        local success, err = turtle.up()
        return { success = success, error = err }
    end,
    ["down"] = function(args)
        local success, err = turtle.down()
        return { success = success, error = err }
    end,
    ["turnLeft"] = function(args)
        local success, err = turtle.turnLeft()
        return { success = success, error = err }
    end,
    ["turnRight"] = function(args)
        local success, err = turtle.turnRight()
        return { success = success, error = err }
    end,

    ["dig"] = function(args)
        local success, err = turtle.dig(args[1])
        return { success = success, error = err }
    end,
    ["digUp"] = function(args)
        local success, err = turtle.digUp(args[1])
        return { success = success, error = err }
    end,
    ["digDown"] = function(args)
        local success, err = turtle.digDown(args[1])
        return { success = success, error = err }
    end,

    ["place"] = function(args)
        local success, err = turtle.place(args[1])
        return { success = success, error = err }
    end,
    ["placeUp"] = function(args)
        local success, err = turtle.placeUp(args[1])
        return { success = success, error = err }
    end,
    ["placeDown"] = function(args)
        local success, err = turtle.placeDown(args[1])
        return { success = success, error = err }
    end,

    ["detect"] = function(args)
        return { success = turtle.detect() }
    end,
    ["detectUp"] = function(args)
        return { success = turtle.detectUp() }
    end,
    ["detectDown"] = function(args)
        return { success = turtle.detectDown() }
    end,

    ["inspect"] = function(args)
        local success, data = turtle.inspect()
        return { success = success, data = data }
    end,
    ["inspectUp"] = function(args)
        local success, data = turtle.inspectUp()
        return { success = success, data = data }
    end,
    ["inspectDown"] = function(args)
        local success, data = turtle.inspectDown()
        return { success = success, data = data }
    end,

    ["select"] = function(args)
        return { success = turtle.select(args[1]) }
    end,
    ["getSelectedSlot"] = function(args)
        return { slot = turtle.getSelectedSlot() }
    end,
    ["getItemCount"] = function(args)
        return { count = turtle.getItemCount(args[1]) }
    end,
    ["getItemSpace"] = function(args)
        return { space = turtle.getItemSpace(args[1]) }
    end,
    ["getItemDetail"] = function(args)
        return { detail = turtle.getItemDetail(args[1], args[2]) }
    end,

    ["drop"] = function(args)
        local success, err = turtle.drop(args[1])
        return { success = success, error = err }
    end,
    ["dropUp"] = function(args)
        local success, err = turtle.dropUp(args[1])
        return { success = success, error = err }
    end,
    ["dropDown"] = function(args)
        local success, err = turtle.dropDown(args[1])
        return { success = success, error = err }
    end,

    ["suck"] = function(args)
        local success, err = turtle.suck(args[1])
        return { success = success, error = err }
    end,
    ["suckUp"] = function(args)
        local success, err = turtle.suckUp(args[1])
        return { success = success, error = err }
    end,
    ["suckDown"] = function(args)
        local success, err = turtle.suckDown(args[1])
        return { success = success, error = err }
    end,

    ["transferTo"] = function(args)
        return { success = turtle.transferTo(args[1], args[2]) }
    end,

    ["compare"] = function(args)
        return { data = turtle.compare() }
    end,
    ["compareUp"] = function(args)
        return { data = turtle.compareUp() }
    end,
    ["compareDown"] = function(args)
        return { data = turtle.compareDown() }
    end,
    ["compareTo"] = function(args)
        return { data = turtle.compareTo(args[1]) }
    end,

    ["getFuelLevel"] = function(args)
        return { level = turtle.getFuelLevel() }
    end,
    ["getFuelLimit"] = function(args)
        return { limit = turtle.getFuelLimit() }
    end,
    ["refuel"] = function(args)
        local success, err = turtle.refuel(args[1])
        return { success = success, error = err }
    end,

    ["equipLeft"] = function(args)
        local success, err = turtle.equipLeft()
        return { success = success, error = err }
    end,
    ["equipRight"] = function(args)
        local success, err = turtle.equipRight()
        return { success = success, error = err }
    end,
    ["getEquippedLeft"] = function(args)
        return { detail = turtle.getEquippedLeft() }
    end,
    ["getEquippedRight"] = function(args)
        return { detail = turtle.getEquippedRight() }
    end,

    ["craft"] = function(args)
        local success, err = turtle.craft(args[1])
        return { success = success, error = err }
    end,

    ["attack"] = function(args)
        local success, err = turtle.attack(args[1])
        return { success = success, error = err }
    end,
    ["attackUp"] = function(args)
        local success, err = turtle.attackUp(args[1])
        return { success = success, error = err }
    end,
    ["attackDown"] = function(args)
        local success, err = turtle.attackDown(args[1])
        return { success = success, error = err }
    end,

    -- Custom directives
    ["start_gps_host"] = function(args, socket, server_req_id)
        -- This starts a blocking job to host GPS
        local success = true

        function host()
            socket.send(encapsulate_data({
                type = "response",
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
                                    error = "GPS is currently running, cannot do anything else."
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
    ["stop_gps_host"] = function(args)
        return { success = false, error = "No GPS running." }
    end,
    ["update_location"] = function(args)
        local location_data = { x = args[1], y = args[2], z = args[3], direction = args[4] }

        local file, err = fs.open("location.json", "w")

        if file then
            file.write(textutils.serializeJSON(location_data))
            file.close()
        end

        return { success = file ~= nil, error = err }
    end,
    ["turtle_init"] = function(args, socket)
        -- Try location with File, then GPS, then manual entry
        local location_data

        sleep(1)
        socket.send(encapsulate_data({
            type = "request",
            oneshot = false,
            req_id = turtle_req_id + 1,
            data = {
                action = "ping",
            }
        }))
        turtle_req_id = turtle_req_id + 1
        print(socket.receive())
        sleep(1)

        if fs.exists("location.json") then
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

        return location_data
    end
}

function encapsulate_data(data)
    return textutils.serializeJSON(data)
end

function decapsulate_data(data)
    local json = textutils.unserializeJSON(data)
    return json
end

function handle_command(socket, data)
    -- This will parse the command, execute it, and send back the result to the server
    local request_id = data["req_id"]
    local oneshot = data["oneshot"] 
    local command = data["data"]
    local action = action_table[command.action]

    if action then
        local result = action(command.args, socket, request_id)

        if result and not oneshot then
            -- This is here so a function can choose whether or not to automatically respond.
            -- This allows complex directives (gps_host) to takeover all behaviors until its done
            socket.send(encapsulate_data({
                type = "response",
                res_id = request_id,
                data = result
            }))
        end
    elseif not oneshot then
        local err = { success = false, error = "Invalid action" }
        
        socket.send(encapsulate_data({
            type = "response",
            res_id = request_id,
            data = err
        }))
    end
end

function main()
    -- Connect to websocket
    local SERVER_URL = "ws://localhost:8080"
    local ws = nil

    local success, err = pcall(function()
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
        while true do
            local message, is_binary = ws.receive()
    
            if message then
                local data = decapsulate_data(message)
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
    end )

    -- Print out the error problem
    if err then
        print(err)
    end

    -- Cleanup main
    if ws then
        ws.close()
    end
end

main()