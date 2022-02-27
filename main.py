def tobtks(tmbin, outfile, tempo=[]):
    index = int.from_bytes(tmbin.read(4), "little")
    print(f"Index of file: {hex(index)}")
    start = int.from_bytes(tmbin.read(4), "little")
    tmbin.read(4) #Ignore assets sub
    tickflow = b""

    # .bin tickflow loop or whatever
    # copied from tickompiler
    while True:
        cmd = tmbin.read(4)
        if cmd == b"\xFE\xFF\xFF\xFF" or cmd == None:
            break
        adj_args = []
        if cmd == b"\xFF\xFF\xFF\xFF":
            amount = int.from_bytes(tmbin.read(4), "little")
            for _ in range(arg_count):
                pass
        tickflow += cmd
        arg_count = (int.from_bytes(cmd, "little") >> 10) & 0xF
        for _ in range(arg_count):
            arg = tmbin.read(4)
            if arg[0:3] in adj_args:
                pass # manage pointer stuff
            tickflow += arg


def unpack(c00, outdir):
    pass

if __name__ == "__main__":
    tobtks(open("test_files/in.bin", "rb"), open("test_files/out.btk", "wb"))