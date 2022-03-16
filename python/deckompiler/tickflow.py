class TickflowCommand:
    def __init__(self, opcode, arg0):
        self.opcode = opcode
        self.arg0 = arg0
    def __str__(self):
        return f"{hex(opcode)}" + ("" if arg0 == 0 else (f"{arg0}" if arg0 < 0xA else f"{hex(arg0)}"))

class StringOperation:
    def __init__(self, command, arguments, scene_id = None):
        self.command = command
        self.arguments = arguments
        if scene_id != None: self.scene_id = scene_id

USTRING_OPS = [
    StringOperation(TickflowCommand(0x31, 0), [1]),
    StringOperation(TickflowCommand(0x35, 0), [1]),
    StringOperation(TickflowCommand(0x39, 0), [1]),
    StringOperation(TickflowCommand(0x3E, 0), [1]),
    StringOperation(TickflowCommand(0x5D, 0), [1]),
    StringOperation(TickflowCommand(0x5D, 2), [0]),
    StringOperation(TickflowCommand(0x61, 2), [0]),
]

ASTRING_OPS = [
    StringOperation(TickflowCommand(0x3B, 0), [2]),
    StringOperation(TickflowCommand(0x67, 1), [1]),
    StringOperation(TickflowCommand(0x93, 0), [2, 3]),
    StringOperation(TickflowCommand(0x94, 0), [1, 2, 3]),
    StringOperation(TickflowCommand(0x95, 0), [1]),
    StringOperation(TickflowCommand(0xB0, 4), [1]),
    StringOperation(TickflowCommand(0xB0, 5), [1]),
    StringOperation(TickflowCommand(0xB0, 6), [1]),
    StringOperation(TickflowCommand(0x66, 0), [1]),
    StringOperation(TickflowCommand(0x65, 1), [1]),
    StringOperation(TickflowCommand(0x68, 1), [1]),
    StringOperation(TickflowCommand(0xAF, 2), [2]),
    StringOperation(TickflowCommand(0xB5, 0), [0]),
    StringOperation(TickflowCommand(0x105, 0), [0], scene_id=0x1),
    StringOperation(TickflowCommand(0x10B, 0), [0], scene_id=0x2C),
]