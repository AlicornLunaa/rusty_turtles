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
end

main()