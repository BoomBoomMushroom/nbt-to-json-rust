import gzip

with gzip.open(input("In File Path: "), "rb") as fIn:
    with open(input("Out File Path: "), "wb") as fOut:
        fOut.write(fIn.read())

print("Done!")