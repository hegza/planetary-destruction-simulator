// Liquid-sim cellular automata based on Kellomäki's work at (link is Tampere
// University of Technology internal)
// https://tutcris.tut.fi/portal/files/4312220/kellomaki_1354.pdf

// Passed in as compile flags:
// DT := fixed_deltatime
// DX := distance between two cell-centers
// DX3 := volume of a cell

// Normalize the marching-cube related ranges from {-1, 1} where mass is -1 and
// no-mass is 1 into {0, 1} where mass is 1 and 0 is no-mass
#define NORMF(x) (((-(x)) + 1.0f) * 0.5f)
// The same, but backwards
#define DENORMF(x) (-((x) * 2.0f - 1.0f))

__kernel void simulate_liquid(
        __global float const * restrict const old_mass,
        __global float3 const * restrict const old_flow,
        __global float const * restrict const old_temp,
        __global float * restrict new_mass,
        __global float3 * restrict new_flow,
        __global float * restrict new_temp) {
    const size_t dim = get_global_size(0);
    const size_t z = get_global_id(0);
    const size_t y = get_global_id(1);
    const size_t x = get_global_id(2);

    const size_t gid = z * dim * dim + y * dim + x;
    // Edge-cases
    if (z == 0 || y == 0 || x == 0 || z == dim-1 || y == dim-1 || x == dim-1) {
        new_mass[gid] = old_mass[gid];
        new_flow[gid] = old_flow[gid];
        new_temp[gid] = old_temp[gid];
    }
    else {
        // Assign all relative indices for the cell
        // (p, n) := (positive, negative)
        const size_t gid_zp = gid + dim * dim;
        const size_t gid_zn = gid - dim * dim;
        const size_t gid_yp = gid + dim;
        const size_t gid_yn = gid - dim;
        const size_t gid_xp = gid + 1;
        const size_t gid_xn = gid - 1;

        // Cross-section of the pipe
        const float A = 0.0001f;
        // A Magical Stabilization Factor (time-dependent < 1)
        const float F_STAB = 1.0f*DT;

        // Temperature loss due to "radiation"
        const float RADIATE = 0.1f;
        // Temperature diffusion
        const float DIFFUSE = 0.1f;


        // Load neighborhood
        const float mass = NORMF(old_mass[gid]);
        const float mass_zp = NORMF(old_mass[gid_zp]);
        const float mass_yp = NORMF(old_mass[gid_yp]);
        const float mass_xp = NORMF(old_mass[gid_xp]);
        /*
        const float temp_zp = old_temp[gid_zp];
        const float temp_yp = old_temp[gid_yp];
        const float temp_xp = old_temp[gid_xp];
        */
        const float3 flow = old_flow[gid];

        // Compute vector to global model center and gravity acceleration
        const float center = (float)(dim-1)/2.0f;
        const float diff_x = (float)center-x;
        const float diff_y = (float)center-y;
        const float diff_z = (float)center-z;
        // to_origo is normalized: len == 1
        const float3 to_origo = normalize((float3)(diff_x, diff_y, diff_z));
        const float dist2 = diff_x*diff_x + diff_y*diff_y + diff_z*diff_z;
        const float3 g_accel = min(to_origo / dist2, 10.0f);

        // Flow update:
        //  Calculate mass differential and increase flow towards origo.
        const float mass_diff_zp = mass_zp - mass;
        const float delta_flow_z = A * mass_diff_zp * g_accel.z * DT / DX;
        new_flow[gid].z = flow.z + delta_flow_z;
        const float mass_diff_yp = mass_yp - mass;
        const float delta_flow_y = A * mass_diff_yp * g_accel.y * DT / DX;
        new_flow[gid].y = flow.y + delta_flow_y;
        const float mass_diff_xp = mass_xp - mass;
        const float delta_flow_x = A * mass_diff_xp * g_accel.x * DT / DX;
        new_flow[gid].x = flow.x + delta_flow_x;

        // Mass update:
        //  Load stored new flows for updating mass.
        __global float3 *zn_flow = &new_flow[gid_zn];
        __global float3 *yn_flow = &new_flow[gid_yn];
        __global float3 *xn_flow = &new_flow[gid_xn];
        __global float3 *p_flow = &new_flow[gid];
        float total_outflow =
            (*p_flow).z + (*zn_flow).z +
            (*p_flow).y + (*yn_flow).y +
            (*p_flow).x + (*xn_flow).x;
        // Limit mass-outflow in case there's not enough mass to go around

        if (total_outflow > mass) {
            // Multiply such that the flow is not allowed to move more than what is left
            const float scale = total_outflow / mass;
            // Scale back each out-flow-component
            (*zn_flow).z = ((*zn_flow).z) / scale;
            (*yn_flow).y = ((*yn_flow).y) / scale;
            (*xn_flow).x = ((*xn_flow).x) / scale;
            (*p_flow).z = ((*p_flow).z) / scale;
            (*p_flow).y = ((*p_flow).y) / scale;
            (*p_flow).x = ((*p_flow).x) / scale;
        }
        // Limit mass-inflow in case there's too much mass
        else if (-total_outflow + mass > 1.0f) {
            // Multiply such that the mass on the next frame will not exceed 1
            const float scale = total_outflow / (1.0f - mass);
            // Scale back each out-flow-component
            (*zn_flow).z = ((*zn_flow).z) / scale;
            (*yn_flow).y = ((*yn_flow).y) / scale;
            (*xn_flow).x = ((*xn_flow).x) / scale;
            (*p_flow).z = ((*p_flow).z) / scale;
            (*p_flow).y = ((*p_flow).y) / scale;
            (*p_flow).x = ((*p_flow).x) / scale;
        }
        float final_outflow =
            (*p_flow).z + (*zn_flow).z +
            (*p_flow).y + (*yn_flow).y +
            (*p_flow).x + (*xn_flow).x;

        new_mass[gid] = DENORMF(mass - DT * final_outflow * F_STAB / DX3);

        /*
        // Temperature update: temperature flows with advection and diffusion and radiates away
        const float RAD_FACTOR = 1.0 - RADIATE*DT;
        const float advection = total_outflow;

        const float diffusion = (
            (temp_zp - old_temp[gid]) +
            (temp_yp - old_temp[gid]) +
            (temp_xp - old_temp[gid])) * DIFFUSE;

        new_temp[gid] = (old_temp[gid] - DT * (advection + diffusion)) * RAD_FACTOR;
        */
    }
}
