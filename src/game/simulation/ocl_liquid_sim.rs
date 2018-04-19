use ocl;
use ocl::{Buffer, Kernel, OclPrm, ProQue, SpatialDims};
use ocl::enums::ArgVal;

/// cell_dist = cell distance in simulation space (0..1)
pub fn bind<T, D>(
    src: &str,
    func: &str,
    mass_bufs: [&[T]; 2],
    // Flows are float3's, not floats
    flow_bufs: [&[T]; 2],
    temp_bufs: [&[T]; 2],
    dims: D,
    fixed_deltatime: f32,
    cell_dist: f32,
) -> (Kernel, [Buffer<T>; 2], [Buffer<T>; 2], [Buffer<T>; 2])
where
    T: OclPrm,
    D: Into<SpatialDims>,
{
    let mut program_builder = ocl::builders::ProgramBuilder::new();
    program_builder
        .source_file(src)
        .cmplr_opt(format!("-D DT={}", fixed_deltatime))
        .cmplr_opt(format!("-D DX={}", cell_dist))
        .cmplr_opt(format!("-D DX3={}", cell_dist * cell_dist * cell_dist));
    let pro_que = ProQue::builder()
        .prog_bldr(program_builder)
        .dims(dims)
        .build()
        .unwrap();

    let mass_buf_0 = unsafe {
        pro_que
            .buffer_builder::<T>()
            .use_host_slice(mass_bufs[0])
            .build()
            .unwrap()
    };
    let mass_buf_1 = unsafe {
        pro_que
            .buffer_builder::<T>()
            .use_host_slice(mass_bufs[1])
            .build()
            .unwrap()
    };

    let flow_buf_0 = unsafe {
        pro_que
            .buffer_builder::<T>()
            .len(flow_bufs[0].len())
            .use_host_slice(flow_bufs[0])
            .build()
            .unwrap()
    };
    let flow_buf_1 = unsafe {
        pro_que
            .buffer_builder::<T>()
            .len(flow_bufs[1].len())
            .use_host_slice(flow_bufs[1])
            .build()
            .unwrap()
    };

    // Temperatures
    let temp_buf_0 = unsafe {
        pro_que
            .buffer_builder::<T>()
            .use_host_slice(temp_bufs[0])
            .build()
            .unwrap()
    };
    let temp_buf_1 = unsafe {
        pro_que
            .buffer_builder::<T>()
            .use_host_slice(temp_bufs[1])
            .build()
            .unwrap()
    };

    let kernel = pro_que
        .kernel_builder(func)
        .arg(&mass_buf_0)
        .arg(&flow_buf_0)
        .arg(&temp_buf_0)
        .arg(&mass_buf_1)
        .arg(&flow_buf_1)
        .arg(&temp_buf_1)
        .build()
        .unwrap();

    (
        kernel,
        [mass_buf_0, mass_buf_1],
        [flow_buf_0, flow_buf_1],
        [temp_buf_0, temp_buf_1],
    )
}

pub fn call<T>(
    kernel: &Kernel,
    mass_bufs: &[Buffer<T>; 2],
    // Flows are float3's, not floats
    flow_bufs: &[Buffer<T>; 2],
    temp_bufs: &[Buffer<T>; 2],
    frame_count: usize,
) where
    T: OclPrm,
{
    // Double buffering!!!
    let even = frame_count % 2;
    let odd = (frame_count + 1) % 2;
    let read_mass_buf = &mass_bufs[even];
    let write_mass_buf = &mass_bufs[odd];
    let read_flow_buf = &flow_bufs[even];
    let write_flow_buf = &flow_bufs[odd];
    let read_temp_buf = &temp_bufs[even];
    let write_temp_buf = &temp_bufs[odd];
    unsafe {
        kernel
            .set_arg_unchecked(0, ArgVal::mem(read_mass_buf))
            .unwrap();
        kernel
            .set_arg_unchecked(1, ArgVal::mem(read_flow_buf))
            .unwrap();
        kernel
            .set_arg_unchecked(2, ArgVal::mem(read_temp_buf))
            .unwrap();
        kernel
            .set_arg_unchecked(3, ArgVal::mem(write_mass_buf))
            .unwrap();
        kernel
            .set_arg_unchecked(4, ArgVal::mem(write_flow_buf))
            .unwrap();
        kernel
            .set_arg_unchecked(5, ArgVal::mem(write_temp_buf))
            .unwrap();
        kernel.enq().unwrap();

        // Mapping causes the buffers to flush to the host pointer
        write_mass_buf.map().read().enq().unwrap();
        write_flow_buf.map().read().enq().unwrap();
        write_temp_buf.map().read().enq().unwrap();
    }
}
