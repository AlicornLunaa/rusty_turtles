local opcodes = require("vturtle.op_codes")

-- Connectivity variables
local SERVER_URL = "ws://localhost:8080"
local ws = nil

-- Spatial variables
local x, y, z = 0, 0, 0
local rotation = 0

-- Virtual Turtle module
local vturtle = {}

-- Connection functions
function vturtle.init()
	-- Attempt to connect to the server
	print("Connecting to server at " .. SERVER_URL)
	ws = http.websocket(SERVER_URL)

	if not ws then
		error("Failed to connect to server")
		return false
	end

	ws.send("turtle")
	print("Connected to server and identified as turtle")

	-- Load spatial variables from file if it exists
	if fs.exists("spatial_vars.txt") then
		local file = fs.open("spatial_vars.txt", "r")
		local data = textutils.unserialize(file.readAll())
		file.close()
		x, y, z = data.x, data.y, data.z
		rotation = data.rotation
		print("Loaded spatial variables from file: x=" .. x .. ", y=" .. y	.. ", z=" .. z .. ", rotation=" .. rotation)
	else
		print("No spatial variables file found, starting with defaults")
	end

	return true
end

function vturtle.cleanup()
	if ws then
		ws.close()
		print("Closed connection to server")
	end
end

function vturtle.is_connected()
	return ws ~= nil
end

function vturtle.update_spatial_vars(new_x, new_y, new_z, new_rotation)
	-- Save the new spatial variables to the local state
	x, y, z = new_x, new_y, new_z
	rotation = new_rotation

	-- Save to a file for persistence
	local file = fs.open("spatial_vars.txt", "w")
	local data = {x = x, y = y, z = z, rotation = rotation}
	file.write(textutils.serialize(data))
	file.close()

	-- Send the updated position and rotation to the server
	if vturtle.is_connected() then
		local update_pos_payload = {opcodes.UPDATE_POSITION, x, y, z}
		local update_rot_payload = {opcodes.UPDATE_ROTATION, rotation}
		ws.send(textutils.serializeJSON(update_pos_payload))
		ws.send(textutils.serializeJSON(update_rot_payload))
	end
end

-- Movement functions
function vturtle.forward()
	if not vturtle.is_connected() then
		error("Not connected to server")
		return false
	end

	-- Move turtle forward
	if not turtle.forward() then
		return false
	end

	-- Calculate new position based on current rotation
	local new_x, new_y, new_z = x, y, z

	if rotation == 0 then
		new_z = z + 1
	elseif rotation == 1 then
		new_x = x + 1
	elseif rotation == 2 then
		new_z = z - 1
	elseif rotation == 3 then
		new_x = x - 1
	end

	-- Update spatial variables and notify server
	vturtle.update_spatial_vars(new_x, new_y, new_z, rotation)
	return true
end

return vturtle
