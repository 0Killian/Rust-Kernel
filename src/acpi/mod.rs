use alloc::vec::Vec;
use ::aml::{AmlName, NamespaceLevel};
use acpi::AcpiTables;
use lazy_static::lazy_static;
use log::{error, info, warn};
pub use crate::acpi::acpi_device::AcpiDevice;
pub use crate::acpi::aml::AML_CONTEXT;
use crate::BOOT_INFO;
use crate::device::Device;
use crate::pci::PCI_HANDLER;
use spin::Mutex;
use crate::acpi::handler::KernelAcpiHandler;

mod handler;
mod aml;
mod acpi_device;
mod acpi_driver;

pub use acpi_driver::AcpiDriver;

pub struct Acpi
{
    pub acpi_tables: AcpiTables<KernelAcpiHandler>,
}

impl Acpi
{
    pub unsafe fn new() -> Self
    {
        let handler = KernelAcpiHandler;
        let rsdp_phys_addr = BOOT_INFO.rsdp_addr.into_option().unwrap();

        let mut acpi = Acpi {
            acpi_tables: AcpiTables::from_rsdp(handler, rsdp_phys_addr as usize).expect("Failed to initialize ACPI")
        };

        info!("Acpi Tables :");
        for sdt in acpi.acpi_tables.sdts.iter() {
            info!("{:#?}", sdt.0);
        }

        acpi
    }

    pub fn enumerate_devices(&mut self) -> Vec<Device>
    {
        info!("[ACPI] Enumerating ACPI devices");
        let acpi_devices = self.enumerate_acpi_devices();
        info!("[ACPI] Enumerated {} ACPI devices", acpi_devices.len());
        info!("[ACPI] Enumerating PCI devices");
        let pci_devices = match &mut *PCI_HANDLER.lock()
        {
            Some(pci) => pci.enumerate_devices(),
            None =>
                {
                    warn!("[ACPI] PCI handler not initialized, skipping PCI enumeration");
                    Vec::new()
                }
        };
        info!("[ACPI] Enumerated {} PCI devices", pci_devices.len());

        let mut devices = Vec::with_capacity(acpi_devices.len() + pci_devices.len());

        for dev in acpi_devices
        {
            devices.push(Device::Acpi(dev));
        }

        for dev in pci_devices
        {
            devices.push(Device::Pci(dev));
        }

        devices
    }

    fn enumerate_acpi_devices(&mut self) -> Vec<AcpiDevice>
    {
        let mut hids_handle = Vec::new();
        if let Err(e) = &AML_CONTEXT.lock().namespace.traverse(&mut |name: &AmlName, level: &NamespaceLevel|
        {
            info!("[AML] Found ACPI Namespace: {} ({} objects)", name.as_string(), level.values.len());

            for value in level.values.iter()
            {
                if value.0.as_str() == "_HID"
                {
                    hids_handle.push(*value.1);
                }
            }

            Ok(true)
        })
        {
            error!("[AML] Failed to traverse namespace: {:?}", e);
            return Vec::new();
        }

        Vec::from_iter(
            hids_handle.iter().map(
                |hid| AcpiDevice::new(
                    AML_CONTEXT.lock().namespace.get(*hid).unwrap().clone()
                )
            ).into_iter()
        )
    }
}

lazy_static!
{
    pub static ref ACPI: Mutex<Acpi> = Mutex::new(unsafe { Acpi::new() });
}

