//! Reads live GPU data from the `libnvidia-ml` shared library.

use crate::error;
use libloading::{Library, Symbol};
use std::{path::Path, process::exit, ptr::null_mut};

type NvmlInit = unsafe extern "C" fn() -> i32;
type NvmlDeviceGetCount = unsafe extern "C" fn(count: *mut u32) -> i32;
type NvmlDeviceGetHandleByIndex = unsafe extern "C" fn(index: u32, device: *mut *mut u8) -> i32;
type NvmlDeviceGetUtilizationRates = unsafe extern "C" fn(device: *mut u8, utilization: *mut Utilization) -> i32;
type NvmlDeviceGetTemperature = unsafe extern "C" fn(device: *mut u8, sensor: u32, temp: *mut u32) -> i32;

#[repr(C)]
struct Utilization {
    gpu: u32,
    memory: u32,
}

const LIB_PATHS: [&str; 4] = [
    "/usr/lib/x86_64-linux-gnu/nvidia/current/libnvidia-ml.so",
    "/usr/lib/libnvidia-ml.so",
    "/usr/lib64/libnvidia-ml.so",
    "/usr/lib32/libnvidia-ml.so",
];

pub struct Gpu {
    lib: Library,
    device: *mut u8,
}

impl Gpu {
    /// Initializes NVML with the first GPU installed in the system.
    pub fn new() -> Self {
        unsafe {
            // Try to open `libnvidia-ml.so` directly, on error use `LIB_PATHS` as fallback
            let lib = Library::new("libnvidia-ml.so").unwrap_or_else(|_| {
                LIB_PATHS
                    .iter()
                    .find_map(|path| {
                        if Path::new(path).exists() {
                            Library::new(path).ok()
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

            // Count devices
            let mut dev_count: u32 = 0;
            let get_device_count: Symbol<NvmlDeviceGetCount> = lib.get(b"nvmlDeviceGetCount").unwrap();
            if get_device_count(&mut dev_count as *mut u32) != 0 || dev_count < 1 {
                error!("No NVIDIA GPU was found");
                exit(1);
            }

            // Get device handle for GPU 0
            let mut device: *mut u8 = null_mut();
            let get_handle: Symbol<NvmlDeviceGetHandleByIndex> = lib.get(b"nvmlDeviceGetHandleByIndex").unwrap();
            if get_handle(0, &mut device as *mut *mut u8) != 0 {
                error!("Failed to get handle for GPU 0 (NVIDIA)");
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
}
