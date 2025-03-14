// gpu.rs

#[cfg(feature = "nvidia-gpu")]
use nvml_wrapper::Nvml;
#[cfg(feature = "nvidia-gpu")]
use std::sync::Arc;

#[cfg(feature = "nvidia-gpu")]
pub struct GpuMonitor {
    // Store nvml in an Arc to handle ownership issues
    nvml: Arc<Nvml>,
}

#[cfg(feature = "nvidia-gpu")]
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
    /// Initialize the GPU monitoring
    pub fn new() -> Result<Self, nvml_wrapper::error::NvmlError> {
        // Create Nvml and wrap it in an Arc
        let nvml = Arc::new(Nvml::init()?);
        
        Ok(GpuMonitor { nvml })
    }
    
    /// Get the number of GPUs
    pub fn device_count(&self) -> Result<usize, nvml_wrapper::error::NvmlError> {
        Ok(self.nvml.device_count()? as usize)
    }
    
    /// Get GPU information
    pub fn get_gpu_info(&self) -> Vec<GpuInfo> {
        let count = match self.nvml.device_count() {
            Ok(count) => count,
            Err(_) => return Vec::new(),
        };
        
        let mut gpu_info = Vec::with_capacity(count as usize);
        
        for i in 0..count {
            let device = match self.nvml.device_by_index(i) {
                Ok(dev) => dev,
                Err(_) => continue,
            };
            
            let name = device.name().unwrap_or_else(|_| "Unknown".into());
            let utilization = device.utilization_rates()
                .map(|util| util.gpu as f32)
                .unwrap_or(0.0);
            let mem_info = device.memory_info().ok();
            let temp = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .unwrap_or(0);
            
            let (total_mem, used_mem, mem_pct) = if let Some(mem) = mem_info {
                let total = mem.total / 1024 / 1024; // Convert to MB
                let used = mem.used / 1024 / 1024;   // Convert to MB
                let pct = (used as f32 / total as f32) * 100.0;
                (total, used, pct)
            } else {
                (0, 0, 0.0)
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
        
        gpu_info
    }
}

#[cfg(not(feature = "nvidia-gpu"))]
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