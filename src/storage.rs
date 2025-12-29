unsafe extern "C" {
    static __userdata_start: u8;
    static __userdata_size: u8;
}

/// Start address of USERDATA memory
#[inline(always)]
fn userdata_start() -> usize {
    &raw const __userdata_start as usize
}

/// Size of USERDATA memory
#[inline(always)]
fn userdata_size() -> usize {
    &raw const __userdata_size as usize
}
