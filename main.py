def tobtks(tmbin, outfile, tempo=[]):
    index = int.from_bytes(tmbin.read(4), "little")
    print(f"Index of file: {hex(index)}")

def unpack(c00, outdir):
    pass

if __name__ == "__main__":
    tobtks(open("test_files/in.bin", "rb"), open("test_files/out.btk", "wb"))