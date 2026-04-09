-- This is a simple dumb terminal program for the turtle.
-- It is intended to be the part of the program which executes commands
-- given from the digital twin in rust.
local action_table = {
    ["forward"] = function(args)
        local success, err = turtle.forward()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["back"] = function(args)
        local success, err = turtle.back()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["up"] = function(args)
        local success, err = turtle.up()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["down"] = function(args)
        local success, err = turtle.down()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["turnLeft"] = function(args)
        local success, err = turtle.turnLeft()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["turnRight"] = function(args)
        local success, err = turtle.turnRight()
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["dig"] = function(args)
        local success, err = turtle.dig(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["digUp"] = function(args)
        local success, err = turtle.digUp(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["digDown"] = function(args)
        local success, err = turtle.digDown(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["place"] = function(args)
        local success, err = turtle.place(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["placeUp"] = function(args)
        local success, err = turtle.placeUp(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["placeDown"] = function(args)
        local success, err = turtle.placeDown(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["detect"] = function(args)
        return textutils.serializeJSON({ success = turtle.detect() })
    end,
    ["detectUp"] = function(args)
        return textutils.serializeJSON({ success = turtle.detectUp() })
    end,
    ["detectDown"] = function(args)
        return textutils.serializeJSON({ success = turtle.detectDown() })
    end,

    ["inspect"] = function(args)
        local success, data = turtle.inspect()
        return textutils.serializeJSON({ success = success, data = data })
    end,
    ["inspectUp"] = function(args)
        local success, data = turtle.inspectUp()
        return textutils.serializeJSON({ success = success, data = data })
    end,
    ["inspectDown"] = function(args)
        local success, data = turtle.inspectDown()
        return textutils.serializeJSON({ success = success, data = data })
    end,

    ["select"] = function(args)
        return textutils.serializeJSON({ success = turtle.select(args[1]) })
    end,
    ["getSelectedSlot"] = function(args)
        return textutils.serializeJSON({ slot = turtle.getSelectedSlot() })
    end,
    ["getItemCount"] = function(args)
        return textutils.serializeJSON({ count = turtle.getItemCount(args[1]) })
    end,
    ["getItemSpace"] = function(args)
        return textutils.serializeJSON({ space = turtle.getItemSpace(args[1]) })
    end,
    ["getItemDetail"] = function(args)
        return textutils.serializeJSON({ detail = turtle.getItemDetail(args[1], args[2]) })
    end,

    ["drop"] = function(args)
        local success, err = turtle.drop(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["dropUp"] = function(args)
        local success, err = turtle.dropUp(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["dropDown"] = function(args)
        local success, err = turtle.dropDown(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["suck"] = function(args)
        local success, err = turtle.suck(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["suckUp"] = function(args)
        local success, err = turtle.suckUp(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["suckDown"] = function(args)
        local success, err = turtle.suckDown(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["transferTo"] = function(args)
        return textutils.serializeJSON({ success = turtle.transferTo(args[1], args[2]) })
    end,

    ["compare"] = function(args)
        return textutils.serializeJSON({ data = turtle.compare() })
    end,
    ["compareUp"] = function(args)
        return textutils.serializeJSON({ data = turtle.compareUp() })
    end,
    ["compareDown"] = function(args)
        return textutils.serializeJSON({ data = turtle.compareDown() })
    end,
    ["compareTo"] = function(args)
        return textutils.serializeJSON({ data = turtle.compareTo(args[1]) })
    end,

    ["getFuelLevel"] = function(args)
        return textutils.serializeJSON({ level = turtle.getFuelLevel() })
    end,
    ["getFuelLimit"] = function(args)
        return textutils.serializeJSON({ limit = turtle.getFuelLimit() })
    end,
    ["refuel"] = function(args)
        local success, err = turtle.refuel(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["equipLeft"] = function(args)
        local success, err = turtle.equipLeft()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["equipRight"] = function(args)
        local success, err = turtle.equipRight()
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["getEquippedLeft"] = function(args)
        return textutils.serializeJSON({ detail = turtle.getEquippedLeft() })
    end,
    ["getEquippedRight"] = function(args)
        return textutils.serializeJSON({ detail = turtle.getEquippedRight() })
    end,

    ["craft"] = function(args)
        local success, err = turtle.craft(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["attack"] = function(args)
        local success, err = turtle.attack(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["attackUp"] = function(args)
        local success, err = turtle.attackUp(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,
    ["attackDown"] = function(args)
        local success, err = turtle.attackDown(args[1])
        return textutils.serializeJSON({ success = success, error = err })
    end,

    ["turtle_init"] = function(args)
        local location_data

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

        return textutils.serializeJSON(location_data)
    end
}

function handle_command(socket, command)
    -- This will parse the command, execute it, and send back the result to the server
    local action = action_table[command.action]

    if action then
        local result = action(command.args)
        socket.send(result)
    else
        socket.send(textutils.serializeJSON({ success = false, error = "Invalid action" }))
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
                print("Received command: " .. message)
                
                local command = textutils.unserializeJSON(message)
                handle_command(ws, command)
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