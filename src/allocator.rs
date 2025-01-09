pub mod linked_list;

// requires that `align` is some power of 2
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
