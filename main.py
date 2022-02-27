def tobtks(tmbin, outfile, tempo=[]):
    #not needed- but nice to print for info purposes
    index = int.from_bytes(tmbin.read(4), "little")
    print(f"Index of file: {hex(index)}")

    start = int.from_bytes(tmbin.read(4), "little")
    tmbin.read(4) #Ignore assets sub
    tickflow = bytearray(b"")

    # .bin tickflow loop or whatever
    # copied from tickompiler, modified so that
    pointers = []
    while True:
        cmd = tmbin.read(4)
        if cmd == b"\xFE\xFF\xFF\xFF": # 0xFFFFFFFE (-2) indicates start of string data
            break
        str_args = [] #strings and tickflow pointers have to be stored separately
        ptr_args = [] #because they're managed differently in btks
        if cmd == b"\xFF\xFF\xFF\xFF": # 0xFFFFFFFF (-1) indicates an 'args' section
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
                pointers.append((len(tickflow), 0))
            elif i in ptr_args:
                pointers.append((len(tickflow), 1))
            tickflow += arg
    strings = tmbin.read()

    stringpos = len(tickflow)

    # fix string pointers - stringpos, etc
    for ptr in pointers:
        if ptr[1] != 0: continue
        str_ptr = int.from_bytes(tickflow[ptr[0]:ptr[0]+4], "little") - stringpos
        tickflow[ptr[0]:ptr[0]+4] = str_ptr.to_bytes(4, "little")
    tmbin.close()

    # put everything into sections
    # 1. FLOW
    section_flow = {
        magic: b"FLOW",
        size: 0xC + len(tickflow),
        start: start,
        tickflow: tickflow
    }
    # 2. PTRO
    ptrbin = b""
    for ptr in pointers:
        ptrbin += ptr[0].to_bytes(4, "little")
        ptrbin += ptr[1]
    section_ptro = {
        magic: b"PTRO",
        size: 0xC + len(ptrbin),
        ptr_amt: len(pointers),
        pointers: ptrbin,
    }
    #TODO: 3. TMPO
    section_tmpo = None # in the future, only make it None if there's no tempos
    # 4. STRD
    section_strd = {
        magic: b"STRD",
        size: 0x8 + len(strings),
        strings: strings
    }

    #finally, the header!
    header = {
        magic: b"BTKS",
        size: 0x10 + section_flow["size"] + section_ptro["size"] + section_strd["size"],
        version: 0, #this is rev0 of the BTKS spec
        section_amt: 3 if section_tmpo == None else 4
    }

    if section_tmpo != None:
        header.size += section_tmpo["size"]

    #TODO: put everything into the outfile


def unpack(c00, outdir):
    pass

if __name__ == "__main__":
    try:
        tmbin = open("test_files/in.bin", "rb")
        tobtks(tmbin, open("test_files/out.btk", "wb"))
    except Exception as e:
        print(f"Exception in byte {hex(tmbin.tell())}:")
        raise e