// gpu.rs - Simplified GPU monitoring with runtime detection

use std::time::{Duration, Instant};

// GPU information structure - consistent regardless of GPU type
#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub name: String,
    pub utilization: f32,
    pub temperature: u32,
    pub total_memory: u64,  // in MB
    pub used_memory: u64,   // in MB
    pub memory_usage: f32,  // percentage
    pub vendor: GpuVendor,
}

// GPU vendor types
#[derive(Clone, Debug, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Other,
    None,
}

// GPU monitoring interface
pub struct GpuMonitor {
    // Cache to prevent excessive polling
    last_refresh: Instant,
    cache_duration: Duration,
    cached_info: Vec<GpuInfo>,
    
    // NVIDIA support if available
    #[cfg(feature = "nvidia-gpu")]
    nvml: Option<nvml_wrapper::Nvml>,
}

impl GpuMonitor {
    /// Initialize the GPU monitoring system with runtime detection
    pub fn new() -> Self {
        // Create a monitor with empty cache
        let mut monitor = GpuMonitor {
            last_refresh: Instant::now() - Duration::from_secs(10), // Force initial refresh
            cache_duration: Duration::from_millis(500),
            cached_info: Vec::new(),
            
            // Try to initialize NVIDIA monitoring if available
            #[cfg(feature = "nvidia-gpu")]
            nvml: None,
        };
        
        // Initialize NVIDIA if available and the feature is enabled
        #[cfg(feature = "nvidia-gpu")]
        {
            match nvml_wrapper::Nvml::init() {
                Ok(nvml) => {
                    // Successfully initialized NVIDIA monitoring
                    monitor.nvml = Some(nvml);
                    println!("NVIDIA GPU monitoring initialized successfully");
                },
                Err(e) => {
                    // NVIDIA monitoring failed to initialize
                    eprintln!("NVIDIA monitoring initialization failed: {:?}", e);
                    eprintln!("GPU statistics will not be available");
                }
            }
        }
        
        // Perform initial refresh to populate cache
        let _ = monitor.refresh_gpu_info();
        
        monitor
    }
    
    /// Check if any GPUs are available
    pub fn has_gpus(&self) -> bool {
        !self.cached_info.is_empty()
    }
    
    /// Get the number of GPUs
    pub fn device_count(&self) -> usize {
        self.cached_info.len()
    }
    
    /// Get GPU information with caching to prevent excessive polling
    pub fn get_gpu_info(&self) -> Vec<GpuInfo> {
        // Check if cache is still valid
        if self.last_refresh.elapsed() < self.cache_duration && !self.cached_info.is_empty() {
            return self.cached_info.clone();
        }
        
        // Need to refresh data
        self.refresh_gpu_info()
    }
    
    /// Internal method to actually fetch GPU data
    fn refresh_gpu_info(&self) -> Vec<GpuInfo> {
        let mut gpu_info = Vec::new();
        
        // Try to get NVIDIA GPU info if available
        #[cfg(feature = "nvidia-gpu")]
        if let Some(nvml) = &self.nvml {
            // Add NVIDIA GPUs if available
            if let Ok(count) = nvml.device_count() {
                for i in 0..count {
                    match nvml.device_by_index(i) {
                        Ok(device) => {
                            // Get GPU name with fallback
                            let name = match device.name() {
                                Ok(name) => name,
                                Err(_) => String::from("Unknown NVIDIA GPU"),
                            };
                            
                            // Get utilization with fallback
                            let utilization = match device.utilization_rates() {
                                Ok(util) => util.gpu as f32,
                                Err(_) => 0.0,
                            };
                            
                            // Get memory info with fallback
                            let (total_mem, used_mem, mem_pct) = match device.memory_info() {
                                Ok(mem) => {
                                    let total = mem.total / 1024 / 1024; // Convert to MB
                                    let used = mem.used / 1024 / 1024;   // Convert to MB
                                    let pct = if total > 0 {
                                        (used as f32 / total as f32) * 100.0
                                    } else {
                                        0.0
                                    };
                                    (total, used, pct)
                                },
                                Err(_) => (0, 0, 0.0),
                            };
                            
                            // Get temperature with fallback
                            let temp = match device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                                Ok(t) => t,
                                Err(_) => 0,
                            };
                            
                            gpu_info.push(GpuInfo {
                                name,
                                utilization,
                                temperature: temp,
                                total_memory: total_mem,
                                used_memory: used_mem,
                                memory_usage: mem_pct,
                                vendor: GpuVendor::Nvidia,
                            });
                        },
                        Err(e) => {
                            eprintln!("Error accessing NVIDIA GPU {}: {:?}", i, e);
                        }
                    }
                }
            }
        }
        
        // In a real implementation with interior mutability, we would update the cache here
        // For this simplified example, we'll just return the data
        gpu_info
    }
}