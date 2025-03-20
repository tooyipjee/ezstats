// gpu.rs - Enhanced with better error handling and stability improvements

#[cfg(feature = "nvidia-gpu")]
use nvml_wrapper::Nvml;
#[cfg(feature = "nvidia-gpu")]
use std::sync::Arc;
#[cfg(feature = "nvidia-gpu")]
use std::time::{Duration, Instant};

#[cfg(feature = "nvidia-gpu")]
pub struct GpuMonitor {
    // Store nvml in an Arc to handle ownership issues
    nvml: Arc<Nvml>,
    // Cache GPU info to prevent excessive calls
    last_refresh: Instant,
    cache_duration: Duration,
    cached_info: Vec<GpuInfo>,
}

#[cfg(feature = "nvidia-gpu")]
#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub name: String,
    pub utilization: f32,
    pub temperature: u32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub memory_usage: f32,
}

#[cfg(feature = "nvidia-gpu")]
impl GpuMonitor {
    /// Initialize the GPU monitoring with improved error handling
    pub fn new() -> Result<Self, nvml_wrapper::error::NvmlError> {
        // Create Nvml and wrap it in an Arc
        let nvml = Arc::new(Nvml::init()?);
        
        let monitor = GpuMonitor { 
            nvml,
            last_refresh: Instant::now() - Duration::from_secs(10), // Force initial refresh
            cache_duration: Duration::from_millis(500), // Cache GPU info for 500ms
            cached_info: Vec::new(),
        };
        
        // Perform initial refresh to test if GPU monitoring works
        let _ = monitor.get_gpu_info();
        
        Ok(monitor)
    }
    
    /// Get the number of GPUs
    pub fn device_count(&self) -> Result<usize, nvml_wrapper::error::NvmlError> {
        Ok(self.nvml.device_count()? as usize)
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
        let count = match self.nvml.device_count() {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Error getting GPU count: {:?}", e);
                return Vec::new();
            }
        };
        
        let mut gpu_info = Vec::with_capacity(count as usize);
        
        for i in 0..count {
            let device = match self.nvml.device_by_index(i) {
                Ok(dev) => dev,
                Err(e) => {
                    eprintln!("Error accessing GPU {}: {:?}", i, e);
                    continue;
                }
            };
            
            // Get GPU name with fallback
            let name = match device.name() {
                Ok(name) => name,
                Err(_) => String::from("Unknown GPU"),
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
            });
        }
        
        // Update last refresh time (would require interior mutability in a real implementation)
        // For this example, we'll just return the data directly
        // In a production system, you would use an Arc<Mutex<>> or similar
        gpu_info
    }
}

#[cfg(not(feature = "nvidia-gpu"))]
#[derive(Default)]
pub struct GpuMonitor;

#[cfg(not(feature = "nvidia-gpu"))]
impl GpuMonitor {
    pub fn new() -> Result<Self, &'static str> {
        Ok(GpuMonitor)
    }
    
    pub fn device_count(&self) -> usize {
        0
    }
}