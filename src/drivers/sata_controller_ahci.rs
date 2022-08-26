use crate::drivers::Driver;
use crate::pci::{PciDevice, PciDriver};

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

#[derive(Debug)]
pub struct SataControllerAhci
{
    pci_device: PciDevice,
    abar: u64,
}

impl PciDriver for SataControllerAhci
{
    fn init(device: PciDevice) -> Result<Self, &'static str>
    {
        Ok(SataControllerAhci {
            pci_device: device,
            abar: 0x00,
        })
    }
}