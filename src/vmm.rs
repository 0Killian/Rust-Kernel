use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
};
use x86_64::structures::paging::RecursivePageTable;

unsafe fn active_level_4_table(recursive_index: u16) -> &'static mut PageTable
{
    // a virtual address must be canonical, meaning the 16 most significant bits must be copies of bit 47
    let sign_bits : u64 = if (recursive_index & 0b100000000000) == 1 { 0o177777 << 48 } else { 0 };
    let recursive_index_u64 = recursive_index as u64;

    let level_4_table_addr = VirtAddr::new(sign_bits | (recursive_index_u64 << 39) | (recursive_index_u64 << 30) | (recursive_index_u64 << 21) | (recursive_index_u64 << 12));

    let level_4_table = &mut *(level_4_table_addr.as_mut_ptr() as *mut PageTable);
    level_4_table
}

pub unsafe fn init(recursive_index: u16) -> RecursivePageTable<'static>
{
    let level_4_table = active_level_4_table(recursive_index);
    RecursivePageTable::new(level_4_table).expect("Failed to create recursive page table")
}