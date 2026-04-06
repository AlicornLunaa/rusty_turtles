local SERVER_URL = "ws://localhost:8080"
local ws = nil

local vturtle = {
	init = function()
		print("Connecting to server at " .. SERVER_URL)
		ws = http.websocket(SERVER_URL)

		if not ws then
			error("Failed to connect to server")
			return false
		end

		ws.send("turtle")
		print("Connected to server and identified as turtle")
		return true
	end,

	cleanup = function()
		if ws then
			ws.close()
			print("Closed connection to server")
		end
	end,

	is_connected = function()
		return ws ~= nil
	end
}

return vturtle
