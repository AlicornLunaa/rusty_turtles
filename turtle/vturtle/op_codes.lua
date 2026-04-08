-- Opcode definitions for vturtle
local op_codes = {
    UPDATE_POSITION = 0,
    UPDATE_ROTATION = 1,
    BLOCK_UPDATE = 2, -- Sent to update the block data on the server
}

return op_codes