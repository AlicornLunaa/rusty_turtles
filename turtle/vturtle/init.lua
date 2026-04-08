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

function vturtle.back()
	if not vturtle.is_connected() then
		error("Not connected to server")
		return false
	end

	-- Move turtle backward
	if not turtle.back() then
		return false
	end

	-- Calculate new position based on current rotation
	local new_x, new_y, new_z = x, y, z

	if rotation == 0 then
		new_z = z - 1
	elseif rotation == 1 then
		new_x = x - 1
	elseif rotation == 2 then
		new_z = z + 1
	elseif rotation == 3 then
		new_x = x + 1
	end

	-- Update spatial variables and notify server
	vturtle.update_spatial_vars(new_x, new_y, new_z, rotation)
	return true
end

function vturtle.up()
	if not vturtle.is_connected() then
		error("Not connected to server")
		return false
	end

	-- Move turtle up
	if not turtle.up() then
		return false
	end

	-- Update spatial variables and notify server
	local new_y = y + 1
	vturtle.update_spatial_vars(x, new_y, z, rotation)
	return true
end

function vturtle.down()
	if not vturtle.is_connected() then
		error("Not connected to server")
		return false
	end

	-- Move turtle down
	if not turtle.down() then
		return false
	end

	-- Update spatial variables and notify server
	local new_y = y - 1
	vturtle.update_spatial_vars(x, new_y, z, rotation)
	return true
end

function vturtle.turn_left()
	if not vturtle.is_connected() then
		error("Not connected to server")
		return false
	end

	-- Turn turtle left
	if not turtle.turnLeft() then
		return false
	end

	-- Update rotation and notify server
	local new_rotation = (rotation - 1) % 4
	vturtle.update_spatial_vars(x, y, z, new_rotation)
	return true
end

function vturtle.turn_right()
	if not vturtle.is_connected() then
		error("Not connected to server")
		return false
	end

	-- Turn turtle right
	if not turtle.turnRight() then
		return false
	end

	-- Update rotation and notify server
	local new_rotation = (rotation + 1) % 4
	vturtle.update_spatial_vars(x, y, z, new_rotation)
	return true
end

function vturtle.scan(direction)
	if not vturtle.is_connected() then
		error("Not connected to server")
		return nil
	end

	direction = direction or "forward"
	local success, data

	if direction == "forward" then
		success, data = turtle.inspect()
	elseif direction == "up" then
		success, data = turtle.inspectUp()
	elseif direction == "down" then
		success, data = turtle.inspectDown()
	else
		error("Invalid scan direction: " .. direction)
		return nil
	end

	if not success then
		return nil
	end

	-- Get block position based on current turtle position and scan direction
	local block_x, block_y, block_z = x, y, z
	if direction == "forward" then
		if rotation == 0 then
			block_z = z + 1
		elseif rotation == 1 then
			block_x = x + 1
		elseif rotation == 2 then
			block_z = z - 1
		elseif rotation == 3 then
			block_x = x - 1
		end
	elseif direction == "up" then
		block_y = y + 1
	elseif direction == "down" then
		block_y = y - 1
	end

	-- Send scan data to server
	local scan_payload = {opcodes.BLOCK_UPDATE, data, block_x, block_y, block_z}
	ws.send(textutils.serializeJSON(scan_payload))

	return data
end

function vturtle.fake_scan(direction)
	-- This function simulates a scan without actually performing it, for testing purposes
	if not vturtle.is_connected() then
		error("Not connected to server")
		return nil
	end

	direction = direction or "forward"
	local data = {
		name = "minecraft:stone",
		state = {
			facing = "north",
			type = "smooth"
		}
	}

	-- Get block position based on current turtle position and scan direction
	local block_x, block_y, block_z = x, y, z
	if direction == "forward" then
		if rotation == 0 then
			block_z = z + 1
		elseif rotation == 1 then
			block_x = x + 1
		elseif rotation == 2 then
			block_z = z - 1
		elseif rotation == 3 then
			block_x = x - 1
		end
	elseif direction == "up" then
		block_y = y + 1
	elseif direction == "down" then
		block_y = y - 1
	end

	-- Send fake scan data to server
	local scan_payload = {opcodes.BLOCK_UPDATE, data.name, block_x, block_y, block_z}
	ws.send(textutils.serializeJSON(scan_payload))

	return data
end

return vturtle
