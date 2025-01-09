struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    pub fn new(size: usize) -> Self {
        Self { size, None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

struct LinkedListAlloc {
    head: ListNode,
}

impl LinkedListAlloc {
    // create empty allocator
    pub const fn new() -> Self {
        Self { ListNode::new(0) }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        todo!();
    }
}
