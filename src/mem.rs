use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

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

// make the public function unsafe so the kernel has to ensure validity of
// memory being passed to it rather than having to waste cycles checking on
// every memory access
pub unsafe fn translate_addr(addr: VirtAddr, phys_mem_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_priv(addr, phys_mem_offset)
}

fn translate_addr_priv(addr: VirtAddr, phys_mem_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    // get the address of the top level page table's physical frame
    let (lvl4_table_frame, _) = Cr3::read();

    // store the offset for each page table level
    let tables_idx = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    // define a pointer to traverse the page table
    let mut frame = lvl4_table_frame;

    for &idx in &tables_idx {
        let virt: VirtAddr = phys_mem_offset + frame.start_address().as_u64();
        let tbl_ptr: *const PageTable = virt.as_ptr();
        let tbl = unsafe { &*tbl_ptr };

        let entry = &tbl[idx];
        // point to the current page table frame
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge Frame Error: Not supported"),
        }
    }

    // return the Level 1 page table frame plus some page offset to get our
    // physical page frame address
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
