use core::{marker::PhantomData, mem, slice};

pub const BOOTLOADER_MAGIC: u32 = 0x36d7_6289;
const MEMORY_MAP_TAG_TYPE: u32 = 6;
const MEMORY_AREA_TYPE_AVAILABLE: u32 = 1;

#[repr(C)]
struct BootInformationHeader {
    // Total size covers the fixed header plus every aligned tag that follows.
    total_size: u32,
    _reserved: u32,
}

#[repr(C)]
struct TagHeader {
    // Every Multiboot2 boot-info tag starts with these two fields.
    typ: u32,
    size: u32,
}

#[repr(C)]
struct MemoryMapTagHeader {
    typ: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryArea {
    base_addr: u64,
    length: u64,
    typ: u32,
    _reserved: u32,
}

impl MemoryArea {
    pub fn start_address(&self) -> u64 {
        self.base_addr
    }

    pub fn size(&self) -> u64 {
        self.length
    }

    pub fn is_available(&self) -> bool {
        self.typ == MEMORY_AREA_TYPE_AVAILABLE
    }
}

pub struct BootInformation {
    start: usize,
    end: usize,
}

impl BootInformation {
    pub unsafe fn load(addr: usize) -> Self {
        // This pointer is bootloader-owned. Read only the fixed-size prefix here
        // so later parsing can stay bounded by the advertised total size.
        let header = unsafe { &*(addr as *const BootInformationHeader) };
        Self {
            start: addr,
            end: addr + header.total_size as usize,
        }
    }

    pub fn start_address(&self) -> usize {
        self.start
    }

    pub fn end_address(&self) -> usize {
        self.end
    }

    pub fn memory_map_tag(&self) -> Option<MemoryMapTag<'_>> {
        // After the boot-info header, tags are stored back-to-back with 8-byte
        // padding between them. We stop at the end tag or when the block ends.
        let mut offset = self.start + mem::size_of::<BootInformationHeader>();

        while offset < self.end {
            let header = unsafe { &*(offset as *const TagHeader) };

            if header.typ == MEMORY_MAP_TAG_TYPE {
                let tag = unsafe { &*(offset as *const MemoryMapTagHeader) };
                let entries_offset = offset + mem::size_of::<MemoryMapTagHeader>();
                let entries_size = header.size as usize - mem::size_of::<MemoryMapTagHeader>();
                let entry_count = entries_size / tag.entry_size as usize;

                return Some(MemoryMapTag {
                    // The parser stays zero-copy here. Higher layers can copy
                    // into owned kernel structures when they need mutation.
                    entries: entries_offset as *const MemoryArea,
                    entry_count,
                    _marker: PhantomData,
                });
            }

            // Type 0 with size 8 is the canonical Multiboot2 end tag.
            if header.typ == 0 && header.size == 8 {
                break;
            }

            offset += align_up(header.size as usize, 8);
        }

        None
    }
}

pub struct MemoryMapTag<'a> {
    entries: *const MemoryArea,
    entry_count: usize,
    _marker: PhantomData<&'a MemoryArea>,
}

impl<'a> MemoryMapTag<'a> {
    pub fn memory_areas(&self) -> &'a [MemoryArea] {
        // Entries are packed contiguously after the memory-map tag header.
        unsafe { slice::from_raw_parts(self.entries, self.entry_count) }
    }
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
