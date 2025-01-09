use super::*;
use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAlloc {
    head: ListNode,
}

impl LinkedListAlloc {
    // create empty allocator
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    // adds freed region in memory to the heap allocator's linked list
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // check if the free region is able to hold a ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // initialize new ListNode
        let mut node = ListNode::new(size);
        // set new node->next = head->next
        node.next = self.head.next.take();
        // create a pointer to the free memory address
        let node_ptr = addr as *mut ListNode;
        // write the node to the memory address
        node_ptr.write(node);
        // set head->next = node
        // places node at the beginning of the list, like pushing onto a stack: LIFO
        self.head.next = Some(&mut *node_ptr);
    }

    // finds a region with the given size and alignment and removes it from the linked list
    // returns -> a reference to the node as well as its start address
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        // start at the list head and iterate through until a suitable node is found
        let mut current = &mut self.head;
        while let Some(ref mut node) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&node, size, align) {
                let next = node.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                // remove node from the free region list
                current.next = next;
                return ret;
            } else {
                // move on to the next memory region
                current = current.next.as_mut().unwrap();
            }
        }

        // if no suitable memory region is found return None
        None
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        // checked_add just makes sure there isn't an overflow of our alloc_end variable and if so it returns an error
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // memory region is too small
            return Err(());
        }

        let overhang = region.end_addr() - alloc_end;
        if overhang > 0 && overhang < mem::size_of::<ListNode>() {
            // memory region does not have enough space to hold a list node
            // used since allocation splits it into a used and free part
            return Err(());
        }

        Ok(alloc_start)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAlloc> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAlloc::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let overhang = region.end_addr() - alloc_end;
            if overhang > 0 {
                allocator.add_free_region(alloc_end, overhang);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAlloc::size_align(layout);
        self.lock().add_free_region(ptr as usize, size);
    }
}
