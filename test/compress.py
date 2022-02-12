import os

RUN_CMD = 'cargo run'

# Util script mean to be used as a generator
# of binary numbers. It should reduce the
# friction of creating binary files with numbers
# within them.

SOURCE = 'data/numbers.data'
RESULTS_FILE = 'data/results.data'

# Numbers which hold 32 bits (4 bytes).
# For the moment we're using numbers that
# are not above 255 bits.
# Also we're using big-endian.

NUMBERS = [
    0x00, 0x00, 0x00, 0x23,  # 35
    0x00, 0x00, 0x00, 0x01,  # 1
    0x00, 0x00, 0x00, 0xF0,  # 240
    0x00, 0x00, 0x00, 0x03,  # 3

    0x00, 0x00, 0x00, 0x22,  # 34
    0x00, 0x00, 0x00, 0x00,  # 0
    0x00, 0x00, 0x00, 0xEF,  # 239
    0x00, 0x00, 0x00, 0x02,  # 2

    0x00, 0x00, 0x00, 0x23,  # 35
    0x00, 0x00, 0x00, 0x01,  # 1
    0x00, 0x00, 0x00, 0xF0,  # 240
    0x00, 0x00, 0x00, 0x03,  # 3
]

def build_source():
    with open(SOURCE, 'wb') as f:
        f.write(bytes(NUMBERS))


def main():
    build_source()

    if os.system(RUN_CMD):
        print('Failed to compile binary!')
        return

    with open(RESULTS_FILE, 'rb') as f:
        results = f.read()

    # We expect
    # - a reference: 0x00 0x00 0x00 0x01
    # - block size (bytes): 0x01
    # - reduced blocks: 0x22 0x00 0xE9 0x02

    print(results == bytes([
        0x04, # compressed numbers per chunk

        0x00, 0x00, 0x00, 0x01,
        0x01,
        0x22, 0x00, 0xEF, 0x02,

        0x00, 0x00, 0x00, 0x00,
        0x01,
        0x22, 0x00, 0xEF, 0x02,

        0x00, 0x00, 0x00, 0x01,
        0x01,
        0x22, 0x00, 0xEF, 0x02,
    ]))


if __name__ == '__main__':
    main()
