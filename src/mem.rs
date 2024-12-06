use x86_64::{structures::paging::PageTable, VirtAddr};

// returns a mutable reference to the active top level (level 4) table
// fn needs to be unsafe b/c the caller needs to gurantee that the complete
// physical memory is mapped to virtual memory at the passed phys_mem_offset
// also make sure this fn is only called once to avoid aliasing `&mut`
// references (*undefined behavior)
pub unsafe fn get_top_pg_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (top_table_frame, _) = Cr3::read();

    let phys = top_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let pg_table: *mut PageTable = virt.as_mut_ptr();

    &mut *pg_table
}
