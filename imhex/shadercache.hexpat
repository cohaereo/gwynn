struct Shader {
    u8 hash[16];
    u32 data_length;
    u8 data[data_length];
};

u32 shader_count @ 0x0;
Shader shader[shader_count] @ $;