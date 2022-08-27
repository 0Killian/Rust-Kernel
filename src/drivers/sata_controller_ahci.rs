use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Deref;
use bit_field::BitField;
use bitfield_struct::bitfield;
use log::info;
use volatile::Volatile;
use x86_64::structures::paging::PageTableFlags;
use x86_64::{PhysAddr, VirtAddr};

use crate::pci::{Bar, BistError, PciDevice, PciDriver, StandardHeader};
use crate::{PCI_HANDLER, VMM};

enum FrameInformationStructure
{
    HostToDevice(HostToDeviceFIS),
    DeviceToHost(DeviceToHostFIS),
    DmaSetup(DmaSetupFIS),
    Data(DataFIS),
    PioSetup(PioSetupFIS),
}

impl FrameInformationStructure
{
    fn new(fis_type: u8) -> FrameInformationStructure
    {
        match fis_type
        {
            0x27 => FrameInformationStructure::HostToDevice(HostToDeviceFIS::new()),
            0x34 => FrameInformationStructure::DeviceToHost(DeviceToHostFIS::new()),
            0x41 => FrameInformationStructure::DmaSetup(DmaSetupFIS::new()),
            0x46 => FrameInformationStructure::Data(DataFIS::new()),
            0x5F => FrameInformationStructure::PioSetup(PioSetupFIS::new()),
            _ => panic!("[SATA] Unsupported FIS type: {}", fis_type)
        }
    }
}

#[repr(C)]
struct HostToDeviceFIS
{
    fis_type: u8,
    pm_port_c: u8,
    command: u8,
    feature_low: u8,

    lba_low: u8,
    lba_mid: u8,
    lba_high: u8,
    device: u8,

    lba_low_exp: u8,
    lba_mid_exp: u8,
    lba_high_exp: u8,
    feature_high: u8,

    count_low: u8,
    count_high: u8,
    icc: u8,
    control: u8,

    reserved: [u8; 4],
}

impl HostToDeviceFIS
{
    fn new() -> HostToDeviceFIS
    {
        HostToDeviceFIS {
            fis_type: 0x27,
            pm_port_c: 0x00,
            command: 0x00,
            feature_low: 0x00,
            lba_low: 0x00,
            lba_mid: 0x00,
            lba_high: 0x00,
            device: 0x00,
            lba_low_exp: 0x00,
            lba_mid_exp: 0x00,
            lba_high_exp: 0x00,
            feature_high: 0x00,
            count_low: 0x00,
            count_high: 0x00,
            icc: 0x00,
            control: 0x00,
            reserved: [0x00; 4],
        }
    }
}

#[repr(C)]
struct DeviceToHostFIS
{
    fis_type: u8,
    pm_port_i: u8,
    status: u8,
    error: u8,

    lba0: u8,
    lba1: u8,
    lba2: u8,
    device: u8,

    lba5: u8,
    lba3: u8,
    lba4: u8,
    reserved2: u8,

    count_low: u8,
    count_high: u8,
    reserved3: [u8; 2],

    reserved4: [u8; 4]
}

impl DeviceToHostFIS
{
    fn new() -> DeviceToHostFIS
    {
        DeviceToHostFIS {
            fis_type: 0x34,
            pm_port_i: 0x00,
            status: 0x00,
            error: 0x00,
            lba0: 0x00,
            lba1: 0x00,
            lba2: 0x00,
            device: 0x00,
            lba5: 0x00,
            lba3: 0x00,
            lba4: 0x00,
            reserved2: 0x00,
            count_low: 0x00,
            count_high: 0x00,
            reserved3: [0x00; 2],
            reserved4: [0x00; 4],
        }
    }
}

#[repr(C)]
struct DataFIS
{
    fis_type: u8,
    pm_port: u8,
    reserved1: u8,
    reserved2: u8,

    data: [u8; 0],
}

impl DataFIS
{
    fn new() -> DataFIS
    {
        DataFIS {
            fis_type: 0x46,
            pm_port: 0x00,
            reserved1: 0x00,
            reserved2: 0x00,
            data: [0x00; 0],
        }
    }
}

#[repr(C)]
struct PioSetupFIS
{
    fis_type: u8,
    pm_port_d_i: u8,
    status: u8,
    error: u8,

    lba0: u8,
    lba1: u8,
    lba2: u8,
    device: u8,

    lba3: u8,
    lba4: u8,
    lba5: u8,
    reserved2: u8,

    count_low: u8,
    count_high: u8,
    reserved3: u8,
    e_status: u8,

    tc: u16,
    reserved4: [u8; 2]
}

impl PioSetupFIS
{
    fn new() -> PioSetupFIS
    {
        PioSetupFIS {
            fis_type: 0x5F,
            pm_port_d_i: 0x00,
            status: 0x00,
            error: 0x00,
            lba0: 0x00,
            lba1: 0x00,
            lba2: 0x00,
            device: 0x00,
            lba3: 0x00,
            lba4: 0x00,
            lba5: 0x00,
            reserved2: 0x00,
            count_low: 0x00,
            count_high: 0x00,
            reserved3: 0x00,
            e_status: 0x00,
            tc: 0x00,
            reserved4: [0x00; 2],
        }
    }
}

#[repr(C)]
struct DmaSetupFIS
{
    fis_type: u8,
    pm_port_d_i_a: u8,
    reserved1: [u8; 2],

    dma_buffer_id: u64,

    reserved3: [u32; 2],

    dma_buffer_offset: u32,

    transfer_count: u32,

    reserved4: u32,
}

impl DmaSetupFIS
{
    fn new() -> DmaSetupFIS
    {
        DmaSetupFIS {
            fis_type: 0x41,
            pm_port_d_i_a: 0x00,
            reserved1: [0x00; 2],
            dma_buffer_id: 0x00,
            reserved3: [0x00; 2],
            dma_buffer_offset: 0x00,
            transfer_count: 0x00,
            reserved4: 0x00,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct HbaMemory
{
    host_capabilities: u32,
    global_host_control: u32,
    interrupt_status: u32,
    port_implemented: u32,
    version: u32,
    command_completion_coalescing_control: u32,
    command_completion_coalescing_ports: u32,
    enclosure_management_location: u32,
    enclosure_management_control: u32,
    host_capabilities_extended: u32,
    handoff_control_status: u32,
    reserved: [u8; 0xA0 - 0x2C],
    vendor_specific: [u8; 0x100 - 0xA0],
    hba_ports: [HbaPort; 32]
}

impl Deref for HbaMemory
{
    type Target = HbaMemory;
    fn deref(&self) -> &Self::Target
    {
        &self
    }
}

#[repr(u32)]
#[derive(Copy, Debug, Clone)]
pub enum PortSignature
{
    Ata = 0x00000101,
    Atapi = 0xEB140101,
    Semb = 0xC33C0101,
    PortMultiplier = 96690101,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct HbaPort
{
    command_list_base_address_low: u32,
    command_list_base_address_high: u32,
    fis_base_address_low: u32,
    fis_base_address_high: u32,
    interrupt_status: u32,
    interrupt_enable: u32,
    command_status: u32,
    _reserved0: u32,
    task_file_data: u32,
    signature: PortSignature,
    sata_status: u32,
    sata_control: u32,
    sata_error: u32,
    sata_active: u32,
    command_issue: u32,
    sata_notification: u32,
    fis_based_control_switch: u32,
    _reserved1: [u32; 11],
    vendor_specific: [u32; 4]
}

#[derive(Debug)]
pub struct SataControllerAhci
{
    pci_device: PciDevice,
    header: StandardHeader,
    abar: &'static mut Volatile<HbaMemory>,
    ports: Vec<u32>,
}

impl PciDriver for SataControllerAhci
{
    fn init(device: PciDevice) -> Result<Self, String>
    {
        // Make self test (BIST)
        if let Err(e) = device.get_header().do_bist(PCI_HANDLER.lock().as_ref().unwrap())
        {
            if e != BistError::NotSupported
            {
                return Err(format!("SATA controller self test failed: {:?}", e));
            }
        }

        let standard_header = StandardHeader::new(device.get_address());

        match standard_header.bar(PCI_HANDLER.lock().as_ref().unwrap(), 5)
        {
            Some(Bar::Memory { base, size, prefetchable}) => {
                let mut controller = SataControllerAhci {
                    pci_device: device,
                    header: standard_header,
                    abar: unsafe {
                        &mut *(VMM.lock().map_region(
                            base,
                            core::mem::size_of::<HbaMemory>() as u64,
                            PageTableFlags::PRESENT | PageTableFlags::NO_CACHE | PageTableFlags::WRITABLE | if prefetchable {PageTableFlags::WRITE_THROUGH} else {PageTableFlags::WRITABLE}
                        ).map_err(|e| format!("Failed to map HBA memory: {:?}", e))?.as_mut_ptr() as *mut Volatile<HbaMemory>)
                    },
                    ports: Vec::new()
                };

                controller.enumerate_ports();

                Ok(controller)
            }
            _ => {
                Err(format!("Failed to get ABAR"))
            }
        }
    }
}

impl SataControllerAhci
{
    fn enumerate_ports(&mut self)
    {
        let mut port_implemented = self.abar.read().port_implemented;
        for i in 0..self.abar.read().host_capabilities.get_bits(0..4)
        {
            if port_implemented.get_bit(i as usize)
            {
                let port = self.abar.read().hba_ports[i as usize];
                if port.sata_status.get_bits(0..3) == 3 && port.sata_status.get_bits(8..11) == 1
                {
                    info!("Found port at {} : {:#?}", i, port);
                    self.ports.push(i);
                }
            }
        }
    }
}