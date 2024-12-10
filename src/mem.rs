use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

// setup a dummy frame allocator structure
pub struct EmptyFrameAllocator;

// implementing the FrameAllocator trait is unsafe since the implementer
// must guarantee that the allocator only returns unused frames
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

// A FrameAllocator that can return usable addresses from the bootloader's
// memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /*
        create a FrameAllocator from the memory map

        This function is unsafe since the caller needs to guarantee that
        the passed memory map is valid. Main requirement is that all frames
        marked USABLE are truly unused and not taken up already
    */
    pub unsafe fn init(mem_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map: mem_map,
            next: 0,
        }
    }

    // get an iterator over all of the frames in the memory map currently
    // marked USABLE
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // first get usable regions
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        // transform into an iterator of frame start addrs by flattening
        // nested structure from Iterator<Item = Iterator<Item = u64>> to
        // Iterator<Item = u64> with flat_map and only stepping by PageSize
        // (4KiB), also no need for alignment or rounding math here since
        // the bootloader ensures that all memory areas are page aligned
        let frame_addrs = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addrs.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    // inefficient since it technically re-generates the Iterator<PhysFrame>
    // on every call, so it would be better to make a 'static one however it
    // isn't possible to store an impl Trait type in a struct currently
    // may work one day with _named existential types_ (READ MORE)
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// initialize a new OffsetPageTable
// must be unsafe because the caller needs to guarantee that the complete
// physical memory is mapped to virtual memory at the passed
// `phys_mem_offset`. Also this should be called only once to avoid aliasing
// `&mut` references (undefined)

// return an OffsetPageTable since the bootloader just maps physical memory
// to some virtual offset at phys_mem_offset. However, since OffsetPageTable
// is just an implementation of the Mapper trait we could also use the other
// types like MappedPageTable (all physical memory is mapped somewhere,
// very flexible) or RecursivePageTable (can be used to access page table
// frames through recursive page tables)
pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let lvl4_table = get_top_pg_table(phys_mem_offset);
    OffsetPageTable::new(lvl4_table, phys_mem_offset)
}

// returns a mutable reference to the active top level (level 4) table
// fn needs to be unsafe b/c the caller needs to gurantee that the complete
// physical memory is mapped to virtual memory at the passed phys_mem_offset
// also make sure this fn is only called once to avoid aliasing `&mut`
// references (*undefined behavior)
// only needs to be used by our init function
unsafe fn get_top_pg_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (top_table_frame, _) = Cr3::read();

    let phys = top_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let pg_table: *mut PageTable = virt.as_mut_ptr();

    &mut *pg_table
}

// creates an example mapping to the given virtual page to the physical
// frame 0xb8000 (VGA Buffer location)
pub fn create_example_mapping(
    pg: Page,
    mapper: &mut OffsetPageTable,
    frame_alloc: &mut impl FrameAllocator<Size4KiB>,
) {
    // import page table flags
    use x86_64::structures::paging::PageTableFlags as Flags;

    // set the physical frame where the memory address will
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    // set the location as in memory and writable
    let flags = Flags::PRESENT | Flags::WRITABLE;

    // extremely unsafe since it requires the caller make sure that the
    // frame is not already in use rather than checking for itself
    let map_to_result = unsafe { mapper.map_to(pg, frame, flags, frame_alloc) };

    // flush the newly mapped page from the TLB
    map_to_result.expect("mapping failed").flush();
}
// adding in #[allow(dead_code)] since we will use the OffsetPageTable type
// created in the init() function to handle translation as it has support
// for huge frames and better error checking going forward

// make the public function unsafe so the kernel has to ensure validity of
// memory being passed to it rather than having to waste cycles checking on
// every memory access
#[allow(dead_code)]
pub unsafe fn translate_addr(addr: VirtAddr, phys_mem_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_priv(addr, phys_mem_offset)
}

#[allow(dead_code)]
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
