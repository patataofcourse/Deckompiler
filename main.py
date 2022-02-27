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
                if anncode in (0, 1): #TODO: manage tickflow vs string
                    str_args.append(ann_arg)
                elif anncode == 2:
                    ptr_args.append(ann_arg)
            cmd
            cmd = tmbin.read(4)
        tickflow += cmd
        arg_count = (int.from_bytes(cmd, "little") >> 10) & 0xF
        for i in range(arg_count):
            arg = tmbin.read(4)
            #TODO: manage those
            if i in str_args:
                print("string detected")
            elif i in ptr_args:
                print("pointer detected")
            tickflow += arg
    stringpos = tmbin.tell() - 12
    print(hex(stringpos))
    strings = tmbin.read()
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