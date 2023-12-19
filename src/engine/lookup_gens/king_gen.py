import struct

OFFSETS = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1)
]

def coord_to_abs(x,y):
    return x + y * 8


with open("king_lookup.bin", "wb") as f:

    for y in range(0, 8):
        for x in range(0, 8):

            bitmap_of_jumps = 0

            for offset in OFFSETS:
                new_pos_x = x + offset[0]
                new_pos_y = y + offset[1]

                if new_pos_x < 0 or new_pos_x >= 8 or new_pos_y < 0 or new_pos_y >= 8:
                    continue

                bitmap_of_jumps |= (1 << coord_to_abs(new_pos_x, new_pos_y))
            
            f.write(struct.pack('<Q', bitmap_of_jumps))
            

