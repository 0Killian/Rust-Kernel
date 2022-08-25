use alloc::boxed::Box;
use alloc::vec::Vec;
use ::aml::{AmlContext, AmlName, DebugVerbosity, NamespaceLevel};
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use log::{error, info};
pub use crate::acpi::acpi_device::AcpiDevice;
use crate::acpi::aml::AmlHandler;
use crate::acpi::handler::RcKernelAcpiHandler;
use crate::BOOT_INFO;
use crate::device::Device;
use crate::pci::PciHandler;

mod handler;
mod aml;
mod acpi_device;

pub struct ACPI
{
    acpi_tables: AcpiTables<RcKernelAcpiHandler>,
    pci_handler: Option<PciHandler>,
    aml_context: AmlContext,
    acpi_handler: RcKernelAcpiHandler,
}

impl ACPI
{
    pub unsafe fn new() -> Self
    {
        let handler = RcKernelAcpiHandler::new();
        let rsdp_phys_addr = BOOT_INFO.rsdp_addr.into_option().unwrap();

        let tables = AcpiTables::from_rsdp(handler.clone(), rsdp_phys_addr as usize).expect("Failed to initialize ACPI");

        let mut acpi = match PciHandler::new(&tables)
        {
            Ok(pci) => {
                ACPI {
                    acpi_tables: tables,
                    aml_context: AmlContext::new(
                        Box::new(AmlHandler::new_with_pci(pci.clone())),
                        DebugVerbosity::None),
                    pci_handler: Some(pci),
                    acpi_handler: handler,
                }
            },
            Err(err) => {
                error!("{:?}", err);
                ACPI {
                    acpi_tables: tables,
                    aml_context: AmlContext::new(Box::new(AmlHandler::new()),DebugVerbosity::None),
                    pci_handler: None,
                    acpi_handler: handler,
                }
            }
        };

        if let Some(dsdt) = &acpi.acpi_tables.dsdt
        {
            info!("[AML] Parsing DSDT");
            let phys_addr = dsdt.address;
            let mapping: PhysicalMapping<RcKernelAcpiHandler, u8> = acpi.acpi_handler.map_physical_region(phys_addr, dsdt.length as usize);
            acpi.aml_context.parse_table(core::slice::from_raw_parts(mapping.virtual_start().as_ptr(), dsdt.length as usize)).expect("[AML] Failed to parse DSDT");
        }

        for ssdt in &acpi.acpi_tables.ssdts
        {
            info!("[AML] Parsing SSDT");
            let phys_addr = ssdt.address;
            let mapping: PhysicalMapping<RcKernelAcpiHandler, u8> = acpi.acpi_handler.map_physical_region(phys_addr, ssdt.length as usize);
            acpi.aml_context.parse_table(core::slice::from_raw_parts(mapping.virtual_start().as_ptr(), ssdt.length as usize)).expect("[AML] Failed to parse SSDT");
        }

        acpi
    }

    pub fn enumerate_devices(&mut self) -> Vec<Device>
    {
        let acpi_devices = self.enumerate_acpi_devices();
        let pci_devices = match self.pci_handler.as_mut()
        {
            Some(pci) => pci.enumerate_devices(),
            None => Vec::new()
        };

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
        if let Err(e) = &self.aml_context.namespace.traverse(&mut |name: &AmlName, level: &NamespaceLevel|
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
            Vec::new()
        }
        else
        {
            Vec::from_iter(
                hids_handle.iter().map(
                    |hid| AcpiDevice::new(
                        self.aml_context.namespace.get(*hid).unwrap().clone()
                    )
                ).into_iter()
            )
        }
    }
}

