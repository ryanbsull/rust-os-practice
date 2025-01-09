use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
pub mod linked_list;
use linked_list::LinkedListAlloc;

// requires that `align` is some power of 2
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}
/*
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
*/
#[global_allocator]
static ALLOCATOR: Locked<LinkedListAlloc> = Locked::new(LinkedListAlloc::new());

pub const HEAP_START: usize = 0x_4444_4444_0000; // VirtAddr where heap starts
pub const HEAP_SIZE: usize = 100 * 1024; // heap size in bytes = 1 MiB

// maps the heap memory range to some physical memory frames
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_alloc: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // generate page range from HEAP_START and HEAP_SIZE
    let pg_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + (HEAP_SIZE - 1) as u64;
        let heap_start_pg = Page::containing_address(heap_start);
        let heap_end_pg = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_pg, heap_end_pg)
    };

    // map each page in pg_range to some physical frame
    for pg in pg_range {
        // allocate the physical frame (or throw an error if impossible)
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        // set the page as present and make it writable
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // map the page to the physical frame allocated
        unsafe {
            mapper.map_to(pg, frame, flags, frame_alloc)?.flush();
        }
    }

    // temporary allocator before making a custom one
    unsafe {
        // must lock it since the LockedHeap class uses a mutex to guarantee
        // thread safety
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

// TODO: implement a custom allocator rather than using the linked_list_allocator crate
pub struct CustomAlloc;

unsafe impl GlobalAlloc for CustomAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("CustomAlloc should not need to be deallocated");
    }
}
