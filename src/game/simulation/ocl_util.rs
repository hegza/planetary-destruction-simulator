use ocl;
use ocl::{Buffer, Kernel, OclPrm, ProQue, SpatialDims};

pub fn bind<T, D>(
    src: &str,
    func: &str,
    host_buffer: &[T],
    dims: D,
    fixed_deltatime: f32,
) -> ocl::Result<(Kernel, Buffer<T>)>
where
    T: OclPrm,
    D: Into<SpatialDims>,
{
    let mut program_builder = ocl::builders::ProgramBuilder::new();
    program_builder
        .source_file(src)
        .cmplr_opt(format!("-D DT={}", fixed_deltatime));
    let pro_que = ProQue::builder()
        .prog_bldr(program_builder)
        .dims(dims)
        .build()?;

    let buffer = unsafe {
        pro_que
            .buffer_builder::<T>()
            .use_host_slice(host_buffer)
            .build()?
    };

    let kernel = pro_que
        .kernel_builder(func)
        .arg(&buffer)
        .arg(0.005f32)
        .build()?;

    Ok((kernel, buffer))
}

pub fn call<T>(kernel: &Kernel, buffer: &Buffer<T>) -> ocl::Result<()>
where
    T: OclPrm,
{
    unsafe {
        kernel.enq()?;
        // Mapping causes the buffer to flush to the host pointer
        buffer.map().read().enq()?;
    }

    Ok(())
}
