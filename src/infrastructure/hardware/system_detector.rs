use crate::domain::hardware_detector::{DetectedHardware, IHardwareDetector};

#[derive(Default)]
pub struct SystemHardwareDetector;

impl SystemHardwareDetector {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> Result<DetectedHardware, String> {
        // 1. Detect CPU via Win32 Registry in-process
        let cpu_name = Self::query_win32_reg_string(
            b"HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0\0",
            b"ProcessorNameString\0",
        ).unwrap_or_else(|| "Generic Multi-Core Processor".to_string());

        // 2. Detect GPU & VRAM via Win32 Registry in-process
        let (gpu_name, gpu_vram_gb) = Self::detect_windows_gpu()
            .unwrap_or((Some("Generic Display Adapter".to_string()), None));

        // 3. Detect RAM via in-process GlobalMemoryStatusEx
        let total_ram_gb = Self::query_windows_ram().unwrap_or(16.0);

        let matched_platform = DetectedHardware::match_platform(
            gpu_vram_gb,
            gpu_name.as_deref(),
            total_ram_gb,
        );

        Ok(DetectedHardware {
            cpu_name,
            total_ram_gb,
            gpu_name,
            gpu_vram_gb,
            os_name: "Windows".to_string(),
            matched_platform,
        })
    }

    #[cfg(target_os = "windows")]
    fn query_win32_reg_string(sub_key: &[u8], value_name: &[u8]) -> Option<String> {
        const HKEY_LOCAL_MACHINE: usize = 0x8000_0002;
        const KEY_READ: u32 = 0x20019;

        unsafe extern "system" {
            fn RegOpenKeyExA(
                h_key: usize,
                lp_sub_key: *const u8,
                ul_options: u32,
                sam_desired: u32,
                phk_result: *mut usize,
            ) -> i32;

            fn RegQueryValueExA(
                h_key: usize,
                lp_value_name: *const u8,
                lp_reserved: *const u32,
                lp_type: *mut u32,
                lp_data: *mut u8,
                lpcb_data: *mut u32,
            ) -> i32;

            fn RegCloseKey(h_key: usize) -> i32;
        }

        let mut h_key: usize = 0;
        let res = unsafe { RegOpenKeyExA(HKEY_LOCAL_MACHINE, sub_key.as_ptr(), 0, KEY_READ, &mut h_key) };
        if res != 0 {
            return None;
        }

        let mut buf = [0u8; 256];
        let mut buf_size: u32 = buf.len() as u32;
        let mut val_type: u32 = 0;

        let query_res = unsafe {
            RegQueryValueExA(
                h_key,
                value_name.as_ptr(),
                std::ptr::null(),
                &mut val_type,
                buf.as_mut_ptr(),
                &mut buf_size,
            )
        };
        unsafe { RegCloseKey(h_key) };

        if query_res == 0 && buf_size > 1 {
            let len = (buf_size as usize).saturating_sub(1);
            let s = String::from_utf8_lossy(&buf[..len]).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_gpu() -> Option<(Option<String>, Option<f32>)> {
        const HKEY_LOCAL_MACHINE: usize = 0x8000_0002;
        const KEY_READ: u32 = 0x20019;

        unsafe extern "system" {
            fn RegOpenKeyExA(h_key: usize, lp_sub: *const u8, opt: u32, sam: u32, res: *mut usize) -> i32;
            fn RegQueryValueExA(h_key: usize, name: *const u8, res: *const u32, ty: *mut u32, data: *mut u8, len: *mut u32) -> i32;
            fn RegCloseKey(h_key: usize) -> i32;
        }

        let mut best_gpu: Option<(String, Option<f32>)> = None;

        for idx in 0..4 {
            let key_str = format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{{4d36e968-e325-11ce-bfc1-08002be10318}}\\000{idx}\0");
            let mut h_key: usize = 0;
            if unsafe { RegOpenKeyExA(HKEY_LOCAL_MACHINE, key_str.as_ptr(), 0, KEY_READ, &mut h_key) } != 0 {
                continue;
            }

            let mut desc_buf = [0u8; 256];
            let mut desc_len: u32 = desc_buf.len() as u32;
            let mut ty: u32 = 0;
            let ok_desc = unsafe {
                RegQueryValueExA(h_key, b"DriverDesc\0".as_ptr(), std::ptr::null(), &mut ty, desc_buf.as_mut_ptr(), &mut desc_len)
            };

            if ok_desc == 0 && desc_len > 1 {
                let desc = String::from_utf8_lossy(&desc_buf[..(desc_len as usize).saturating_sub(1)]).trim().to_string();
                if !desc.to_lowercase().contains("basic") && !desc.to_lowercase().contains("mirror") {
                    let mut vram_bytes: u64 = 0;
                    let mut vram_len: u32 = 8;
                    let mut vram_ty: u32 = 0;
                    let vram_gb = if unsafe {
                        RegQueryValueExA(h_key, b"HardwareInformation.qwMemorySize\0".as_ptr(), std::ptr::null(), &mut vram_ty, &mut vram_bytes as *mut u64 as *mut u8, &mut vram_len)
                    } == 0 && vram_bytes > 0 {
                        let gb = vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                        Some(((gb * 10.0).round() / 10.0) as f32)
                    } else {
                        None
                    };

                    let is_better = match (&best_gpu, vram_gb) {
                        (None, _) => true,
                        (Some((_, prev_vram)), Some(cur_vram)) => cur_vram > prev_vram.unwrap_or(0.0),
                        _ => false,
                    };

                    if is_better {
                        best_gpu = Some((desc, vram_gb));
                    }
                }
            }
            unsafe { RegCloseKey(h_key) };
        }

        best_gpu.map(|(d, v)| (Some(d), v))
    }

    #[cfg(target_os = "windows")]
    fn query_windows_ram() -> Option<f32> {
        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }

        unsafe extern "system" {
            fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
        }

        let mut mem = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            dw_memory_load: 0,
            ull_total_phys: 0,
            ull_avail_phys: 0,
            ull_total_page_file: 0,
            ull_avail_page_file: 0,
            ull_total_virtual: 0,
            ull_avail_virtual: 0,
            ull_avail_extended_virtual: 0,
        };

        let ok = unsafe { GlobalMemoryStatusEx(&mut mem) };
        if ok != 0 {
            let gb = mem.ull_total_phys as f64 / (1024.0 * 1024.0 * 1024.0);
            return Some((gb * 10.0).round() as f32 / 10.0);
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn detect_fallback() -> Result<DetectedHardware, String> {
        Ok(DetectedHardware {
            cpu_name: "Generic 8-Core Processor".to_string(),
            total_ram_gb: 16.0,
            gpu_name: Some("Generic GPU (8 GB)".to_string()),
            gpu_vram_gb: Some(8.0),
            os_name: "Unix / POSIX".to_string(),
            matched_platform: crate::domain::hardware::HardwarePlatform::Gpu8Gb,
        })
    }
}

impl IHardwareDetector for SystemHardwareDetector {
    fn detect(&self) -> Result<DetectedHardware, String> {
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self::detect_fallback()
        }
    }
}
