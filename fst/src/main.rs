const FST_HDR_SIM_VERSION_SIZE: usize = 1;
const FST_HDR_DATE_SIZE: usize = 1;
struct FstHeader {
    start_time: u64,
    end_time: u64,
    double_endian_match: bool,
    mem_used_by_writer: u64,
    scope_count: u64,
    var_count: u64,
    max_handle: u64,
    // num_alias = var_count - max_handle
    vc_section_count: u64,
    timescale: i8,
    version: [u8; FST_HDR_SIM_VERSION_SIZE],
    date: [u8; FST_HDR_DATE_SIZE],
    file_type: u8,
    time_zero: u64,
}

struct FstVCData {}

enum FstBlock {
    Header(FstHeader),
    VCData(FstVCData),
}

fn main() {
    println!("Hello, world!");
}
