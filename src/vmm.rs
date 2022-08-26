use core::ops::DerefMut;
use x86_64::{PhysAddr, structures::paging::PageTable, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, RecursivePageTable, Size4KiB, Translate};
use x86_64::structures::paging::mapper::{MapperFlush, MapToError, UnmapError};
use crate::pmm::PMM;
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;
use x86_64::structures::paging::page::PageRangeInclusive;
use crate::allocator::{HEAP_SIZE, HEAP_START};
use crate::BOOT_INFO;

pub struct Vmm
{
    mapper: RecursivePageTable<'static>
}

#[derive(Debug)]
pub enum MappingError
{
    MapToError(MapToError<Size4KiB>),
    UnmapError(UnmapError),
    NoFreePages
}

impl From<MapToError<Size4KiB>> for MappingError
{
    fn from(error: MapToError<Size4KiB>) -> Self
    {
        MappingError::MapToError(error)
    }
}

impl From<UnmapError> for MappingError
{
    fn from(error: UnmapError) -> Self
    {
        MappingError::UnmapError(error)
    }
}

impl Vmm
{
    unsafe fn active_level_4_table(recursive_index: u16) -> &'static mut PageTable
    {
        // a virtual address must be canonical, meaning the 16 most significant bits must be copies of bit 47
        let sign_bits: u64 = if (recursive_index & 0b100000000000) == 1 { 0o177777 << 48 } else { 0 };
        let recursive_index_u64 = recursive_index as u64;

        let level_4_table_addr = VirtAddr::new(sign_bits | (recursive_index_u64 << 39) | (recursive_index_u64 << 30) | (recursive_index_u64 << 21) | (recursive_index_u64 << 12));

        let level_4_table = &mut *(level_4_table_addr.as_mut_ptr() as *mut PageTable);
        level_4_table
    }

    pub unsafe fn new() -> Vmm
    {
        let level_4_table = Self::active_level_4_table(BOOT_INFO.recursive_index.into_option().unwrap());
        Vmm {
            mapper: RecursivePageTable::new(level_4_table).expect("Failed to create recursive page table")
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub unsafe fn map_to(&mut self, page: Page<Size4KiB>,
                                frame: PhysFrame<Size4KiB>,
                                flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    {
        self.mapper.map_to(page, frame, flags, PMM.lock().deref_mut())
    }

    #[inline]
    #[allow(dead_code)]
    pub unsafe fn unmap(&mut self, page: Page) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError>
    {
        self.mapper.unmap(page)
    }

    #[inline]
    pub unsafe fn map(&mut self, page: Page<Size4KiB>, flags: PageTableFlags) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    {
        let frame = PMM.lock().allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        self.mapper.map_to(page, frame, flags, PMM.lock().deref_mut())
    }

    #[inline]
    pub fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr>
    {
        self.mapper.translate_addr(addr)
    }

    fn range_inclusive(start: VirtAddr, end: VirtAddr) -> PageRangeInclusive<Size4KiB>
    {
        let page_start = Page::containing_address(start);
        let page_end = Page::containing_address(end);
        Page::range_inclusive(page_start, page_end)
    }

    pub fn map_region(&mut self, phys_addr: PhysAddr, size: u64, flags: PageTableFlags) -> Result<VirtAddr, MappingError>
    {
        let phys_addr_aligned = phys_addr.align_down(0x1000 as u64);
        let end_phys_addr_aligned = (phys_addr + size).align_up(0x1000 as u64);
        let offset = phys_addr.as_u64() % 0x1000;
        let page_count = (end_phys_addr_aligned - phys_addr_aligned) / 0x1000;

        if let Some(virt_addr) = self.find_free_pages(
            Page::containing_address(VirtAddr::new((HEAP_START + HEAP_SIZE) as u64)),
            Page::containing_address(VirtAddr::new(u64::MAX)),
            page_count as usize
        )
        {
            for i in 0..page_count {
                unsafe {
                    match self.mapper.map_to(
                        Page::containing_address(virt_addr + i * 0x1000),
                        PhysFrame::containing_address(phys_addr_aligned + i * 0x1000),
                        flags,
                        PMM.lock().deref_mut()
                    )
                    {
                        Ok(flush) => flush.flush(),
                        Err(error) => return Err(MappingError::MapToError(error))
                    };
                }
            }

            Ok(virt_addr + offset)
        }
        else
        {
            Err(MappingError::NoFreePages)
        }
    }

    pub fn unmap_region(&mut self, virt_addr: VirtAddr, size: u64) -> Result<(), UnmapError>
    {
        let virt_addr_aligned = virt_addr.align_down(0x1000 as u64);
        let virt_offset = virt_addr.as_u64() % 0x1000;

        let page_range = Self::range_inclusive(
            virt_addr_aligned,
            virt_addr_aligned + size + virt_offset - 1u64
        );

        for page in page_range {
            self.mapper.unmap(page)?.1.flush();
        }

        Ok(())
    }

    pub fn remap_region(&mut self, phys_addr: PhysAddr, virt_addr: VirtAddr, size: u64, flags: PageTableFlags) -> Result<(), MappingError>
    {
        let phys_addr_aligned = phys_addr.align_down(0x1000 as u64);
        let virt_addr_aligned = virt_addr.align_down(0x1000 as u64);

        let phys_offset = phys_addr.as_u64() - phys_addr_aligned.as_u64();
        let virt_offset = virt_addr.as_u64() - virt_addr_aligned.as_u64();

        assert_eq!(phys_offset, virt_offset, "Physical and virtual addresses' offset from nearest page boundary must be equal");

        let page_range = Self::range_inclusive(
            virt_addr_aligned,
            virt_addr_aligned + size + virt_offset - 1u64
        );

        for (i , page) in page_range.enumerate() {
            if let Some(phys) = self.translate_addr(page.start_address())
            {
                if phys == phys_addr_aligned + i * 0x1000
                {
                    continue;
                }
                else
                {
                    self.mapper.unmap(Page::<Size4KiB>::containing_address(virt_addr_aligned + i * 0x1000))?.1.flush();
                }
            }
            unsafe {
                self.mapper.map_to(page, PhysFrame::containing_address(phys_addr_aligned + i as u64 * 0x1000), flags, PMM.lock().deref_mut())?.flush();
            }
        }

        Ok(())
    }

    pub fn find_free_pages(&mut self, start_page: Page<Size4KiB>, end_page: Page<Size4KiB>, count: usize) -> Option<VirtAddr>
    {
        let page_range_inclusive = Page::range_inclusive(start_page, end_page);

        let mut free_page = None;
        let mut page_count = 0;

        for page in page_range_inclusive
        {
            if self.translate_addr(page.start_address()).is_none()
            {
                if free_page.is_none()
                {
                    free_page = Some(page.start_address());
                }

                page_count += 1;
                if count == page_count
                {
                    break;
                }
            }
            else
            {
                if free_page.is_some()
                {
                    free_page = None;
                }
            }
        }

        if page_count == count
        {
            free_page
        }
        else
        {
            None
        }
    }
}

lazy_static! {
    pub static ref VMM: Mutex<Vmm> = unsafe { Mutex::new(Vmm::new()) };
}