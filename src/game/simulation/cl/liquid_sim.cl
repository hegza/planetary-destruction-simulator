// #define DT = fixed_deltatime, compile flag

__kernel void add(__global float* buffer, float scalar) {
    size_t dim = get_global_size(0);
    size_t z = get_global_id(0);
    size_t y = get_global_id(1);
    size_t x = get_global_id(2);
    buffer[z * dim * dim + y * dim + x] -= scalar * DT;
}
