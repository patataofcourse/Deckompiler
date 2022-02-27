def tobtks(tmbin, outfile, tempo=[]):
    index = int.from_bytes(tmbin.read(4), "little")
    print(f"Index of file: {hex(index)}")
    start = int.from_bytes(tmbin.read(4), "little")
    tmbin.read(4) #Ignore assets sub
    tickflow = bytearray(b"")

    # .bin tickflow loop or whatever
    # copied from tickompiler
    ptro_section = []
    while True:
        cmd = tmbin.read(4)
        if cmd == b"\xFE\xFF\xFF\xFF":
            break
        str_args = []
        ptr_args = []
        if cmd == b"\xFF\xFF\xFF\xFF":
            amount = int.from_bytes(tmbin.read(4), "little")
            for _ in range(amount):
                ann = tmbin.read(4)
                anncode = ann[0]
                ann_arg = int.from_bytes(ann[1:], "little")
                if anncode == 0:
                    ptr_args.append(ann_arg)
                elif anncode in (1, 2):
                    str_args.append(ann_arg)
            cmd
            cmd = tmbin.read(4)
        tickflow += cmd
        arg_count = (int.from_bytes(cmd, "little") >> 10) & 0xF
        for i in range(arg_count):
            arg = tmbin.read(4)
            if i in str_args:
                ptro_section.append((len(tickflow), 0))
            elif i in ptr_args:
                ptro_section.append((len(tickflow), 1))
            tickflow += arg
    strings = tmbin.read()

    stringpos = len(tickflow)
    # fix string pointers - stringpos, etc
    for ptr in ptro_section:
        if ptr[1] != 0: continue
        str_ptr = int.from_bytes(tickflow[ptr[0]:ptr[0]+4], "little") - stringpos
        tickflow[ptr[0]:ptr[0]+4] = str_ptr.to_bytes(4, "little")
    tmbin.close()


def unpack(c00, outdir):
    pass

if __name__ == "__main__":
    try:
        tmbin = open("test_files/in.bin", "rb")
        tobtks(tmbin, open("test_files/out.btk", "wb"))
    except Exception as e:
        print(f"Exception in byte {hex(tmbin.tell())}:")
        raise e