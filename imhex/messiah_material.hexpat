import type.magic;

struct String {
    u16 length [[hidden]];
    char string[length];
} [[inline]];

struct MessiahHeader {
    type::Magic<".MESSIAH"> magic;
    u32 type;
};

struct Material {
    String id1;
    String id2;
    u32 unk;
    String dat;
};

MessiahHeader header @ 0;
Material mat @ 0xf;