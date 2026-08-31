//! Reads live GPU data from the `libnvidia-ml` shared library.

use crate::error;
use libloading::{Library, Symbol};
use std::{
    ffi::{c_char, CString},
    path::Path,
    process::exit,
    ptr::null_mut,
};

type NvmlInit = unsafe extern "C" fn() -> u16;
type NvmlDeviceGetHandleByPciBusId = unsafe extern "C" fn(pci_bus_id: *const c_char, device: *mut *mut u8) -> u16;
type NvmlDeviceGetUtilizationRates = unsafe extern "C" fn(device: *mut u8, utilization: *mut Utilization) -> u16;
type NvmlDeviceGetTemperature = unsafe extern "C" fn(device: *mut u8, sensor: u32, temp: *mut u32) -> u16;
type NvmlDeviceGetPowerUsage = unsafe extern "C" fn(device: *mut u8, power: *mut u32) -> u16;
type NvmlDeviceGetClockInfo = unsafe extern "C" fn(device: *mut u8, clock_type: u32, clock: *mut u32) -> u16;

#[repr(C)]
struct Utilization {
    gpu: u32,
    memory: u32,
}

const LIB_PATHS: [&str; 12] = [
    "/usr/lib/x86_64-linux-gnu/nvidia/current/libnvidia-ml.so",
    "/usr/lib/x86_64-linux-gnu/nvidia/current/libnvidia-ml.so.1",
    "/usr/lib/x86_64-linux-gnu/libnvidia-ml.so",
    "/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1",
    "/usr/lib/libnvidia-ml.so",
    "/usr/lib/libnvidia-ml.so.1",
    "/usr/lib64/libnvidia-ml.so",
    "/usr/lib64/libnvidia-ml.so.1",
    "/usr/lib32/libnvidia-ml.so",
    "/usr/lib32/libnvidia-ml.so.1",
    "/run/opengl-driver/lib/libnvidia-ml.so",
    "/run/opengl-driver/lib/libnvidia-ml.so.1",
];

unsafe fn nvml_device_get_handle_by_pci_bus_id(
    get_handle: NvmlDeviceGetHandleByPciBusId,
    pci_address: &str,
    device: *mut *mut u8,
) -> u16 {
    let pci_bus_id = CString::new(pci_address).expect("PCI address contains an embedded NUL byte");

    unsafe { get_handle(pci_bus_id.as_ptr(), device) }
}

pub struct Gpu {
    lib: Library,
    device: *mut u8,
}

impl Gpu {
    /// Initializes NVML with the GPU specified by its PCI address.
    pub fn new(pci_address: &str) -> Self {
        unsafe {
            // Try to open `libnvidia-ml.so` directly, on error use `LIB_PATHS` as fallback
            let lib = Library::new("libnvidia-ml.so").unwrap_or_else(|_| {
                LIB_PATHS
                    .iter()
                    .find_map(|path| {
                        if Path::new(path).exists() {
                            Library::new(*path).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        error!("NVIDIA GPU library was not found");
                        exit(1);
                    })
            });

            // Initialize the library
            let init: Symbol<NvmlInit> = lib.get(b"nvmlInit_v2").unwrap();
            if init() != 0 {
                error!("Failed to initialize NVML");
                exit(1);
            }

            // Get device handle at the specified PCI address
            let mut device: *mut u8 = null_mut();
            let get_handle: Symbol<NvmlDeviceGetHandleByPciBusId> =
                lib.get(b"nvmlDeviceGetHandleByPciBusId_v2").unwrap();
            if nvml_device_get_handle_by_pci_bus_id(*get_handle, pci_address, &mut device as *mut *mut u8) != 0 {
                error!(format!("Failed access GPU (NVIDIA) PCI_ADDR={pci_address}"));
                exit(1);
            }

            Gpu { lib, device }
        }
    }

    /// Reads the GPU temperature from the API and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        let mut temp: u32 = 0;
        unsafe {
            let get_temp: Symbol<NvmlDeviceGetTemperature> = self.lib.get(b"nvmlDeviceGetTemperature").unwrap();
            if get_temp(self.device, 0, &mut temp as *mut u32) != 0 {
                error!("Failed to get GPU temperature (NVIDIA)");
                exit(1);
            }
        }
        if fahrenheit {
            temp = (temp as f32 * 9.0 / 5.0 + 32.0).round() as u32;
        }

        temp as u8
    }

    /// Reads the GPU utilization from the API.
    pub fn get_usage(&self) -> u8 {
        let mut utilization = Utilization { gpu: 0, memory: 0 };
        unsafe {
            let get_usage: Symbol<NvmlDeviceGetUtilizationRates> =
                self.lib.get(b"nvmlDeviceGetUtilizationRates").unwrap();
            if get_usage(self.device, &mut utilization as *mut Utilization) != 0 {
                error!("Failed to get GPU usage (NVIDIA)");
                exit(1);
            }
        }

        utilization.gpu as u8
    }

    /// Reads the GPU power consumption from the API.
    pub fn get_power(&self) -> u16 {
        let mut power: u32 = 0;
        unsafe {
            let get_power: Symbol<NvmlDeviceGetPowerUsage> = self.lib.get(b"nvmlDeviceGetPowerUsage").unwrap();
            if get_power(self.device, &mut power as *mut u32) != 0 {
                error!("Failed to get GPU power (NVIDIA)");
                exit(1);
            }
        }

        (power as f32 / 1000.0).round() as u16
    }

    /// Reads the GPU core frequency from the API.
    pub fn get_frequency(&self) -> u16 {
        let mut clock: u32 = 0;
        unsafe {
            let get_clock: Symbol<NvmlDeviceGetClockInfo> = self.lib.get(b"nvmlDeviceGetClockInfo").unwrap();
            if get_clock(self.device, 0, &mut clock as *mut u32) != 0 {
                error!("Failed to get GPU core frequency (NVIDIA)");
                exit(1);
            }
        }

        clock as u16
    }
}

#[cfg(test)]
mod tests {
    use super::nvml_device_get_handle_by_pci_bus_id;
    use std::{ffi::c_char, ptr::null_mut, slice, str};

    const EXPECTED_PCI_BUS_ID: &[u8] = b"0000:06:00.0\0";
    const PCI_BUS_ID_WITH_SENTINEL: &[u8] = b"0000:06:00.0x";

    unsafe extern "C" fn validate_pci_bus_id(pci_bus_id: *const c_char, _device: *mut *mut u8) -> u16 {
        let pci_bus_id = unsafe { slice::from_raw_parts(pci_bus_id.cast::<u8>(), EXPECTED_PCI_BUS_ID.len()) };

        if pci_bus_id == EXPECTED_PCI_BUS_ID {
            0
        } else {
            1
        }
    }

    #[test]
    fn passes_null_terminated_pci_bus_id_to_nvml() {
        let pci_address = str::from_utf8(&PCI_BUS_ID_WITH_SENTINEL[..EXPECTED_PCI_BUS_ID.len() - 1]).unwrap();
        let mut device: *mut u8 = null_mut();

        let result = unsafe { nvml_device_get_handle_by_pci_bus_id(validate_pci_bus_id, pci_address, &mut device) };

        assert_eq!(result, 0);
    }
}
