const FRAME_SIZE: usize = 4096;

pub struct FrameAllocator {
    curr: usize,
    end: usize,
}

impl FrameAllocator {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            curr: Self::align_up(start, FRAME_SIZE),
            end: Self::align_down(end, FRAME_SIZE),
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
           if self.curr + FRAME_SIZE > self.end {
            None
        } else {
            let frame = self.curr;
            self.curr += FRAME_SIZE;
            Some(frame)
        }
}

    fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }

    fn align_down(addr: usize, align: usize) -> usize {
        addr & !(align - 1)
    }
}