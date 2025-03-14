// system_monitor.rs
// A simple, lightweight system monitoring tool written in Rust that displays
// real-time CPU, RAM, and GPU usage statistics in the terminal with visual widgets.
use sysinfo::{System, SystemExt, CpuExt};
use std::{io, thread, time::Duration};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetForegroundColor},
};

mod gpu;
mod widget;  // Add the widget module
#[cfg(feature = "apple-gpu")]
mod mac_gpu;  // Add the macOS GPU module

use gpu::GpuMonitor;
use widget::{Widget, BarChart};  // Import the widget traits

#[cfg(feature = "nvidia-gpu")]
use gpu::GpuInfo;
#[cfg(feature = "apple-gpu")]
use mac_gpu::{MacGpuMonitor, MacGpuInfo};

// No lifetime required anymore
struct SystemMonitor {
    system: System,
    refresh_rate: Duration,
    #[cfg(feature = "nvidia-gpu")]
    gpu_monitor: Option<GpuMonitor>,
    #[cfg(feature = "apple-gpu")]
    mac_gpu_monitor: Option<MacGpuMonitor>,
}

impl SystemMonitor {
    /// Create a new SystemMonitor with the given refresh rate in milliseconds
    fn new(refresh_ms: u64) -> Self {
        let mut system = System::new_all();
        // Initial system info refresh
        system.refresh_all();
        
        #[cfg(feature = "nvidia-gpu")]
        let gpu_monitor = match GpuMonitor::new() {
            Ok(monitor) => Some(monitor),
            Err(e) => {
                eprintln!("Failed to initialize NVIDIA GPU monitoring: {:?}", e);
                None
            }
        };
        
        #[cfg(feature = "apple-gpu")]
        let mac_gpu_monitor = match MacGpuMonitor::new() {
            Ok(monitor) => Some(monitor),
            Err(e) => {
                eprintln!("Failed to initialize Apple GPU monitoring: {}", e);
                None
            }
        };
        
        SystemMonitor {
            system,
            refresh_rate: Duration::from_millis(refresh_ms),
            #[cfg(feature = "nvidia-gpu")]
            gpu_monitor,
            #[cfg(feature = "apple-gpu")]
            mac_gpu_monitor,
        }
    }
    
    /// Refresh all system information
    fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    /// Get CPU usage as a percentage for each core and overall
    fn get_cpu_usage(&self) -> (Vec<f32>, f32) {
        let per_cpu: Vec<f32> = self.system.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();
            
        let overall_usage = per_cpu.iter().sum::<f32>() / per_cpu.len() as f32;
        
        (per_cpu, overall_usage)
    }
    
    /// Get memory information in MB
    fn get_memory_info(&self) -> (u64, u64, f32) {
        let total_mem = self.system.total_memory() / 1024 / 1024; // Convert to MB
        let used_mem = self.system.used_memory() / 1024 / 1024;   // Convert to MB
        let mem_usage_pct = (used_mem as f32 / total_mem as f32) * 100.0;
        
        (total_mem, used_mem, mem_usage_pct)
    }
    
    /// Get NVIDIA GPU information
    #[cfg(feature = "nvidia-gpu")]
    fn get_gpu_info(&self) -> Vec<GpuInfo> {
        if let Some(gpu_monitor) = &self.gpu_monitor {
            gpu_monitor.get_gpu_info()
        } else {
            Vec::new()
        }
    }
    
    /// Get Apple GPU information
    #[cfg(feature = "apple-gpu")]
    fn get_mac_gpu_info(&self) -> Vec<MacGpuInfo> {
        if let Some(gpu_monitor) = &self.mac_gpu_monitor {
            gpu_monitor.get_gpu_info()
        } else {
            Vec::new()
        }
    }
    
    /// Display system information in the terminal
    fn display(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        
        loop {
            // Refresh system data
            self.refresh();
            
            // Get system metrics
            let (cpu_per_core, cpu_overall) = self.get_cpu_usage();
            let (total_mem, used_mem, mem_usage) = self.get_memory_info();
            
            // Clear screen and reset cursor
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
            
            // Display CPU information
            execute!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print("=== CPU USAGE ===\n"),
                ResetColor
            )?;
            
            // Draw overall CPU usage bar chart
            let cpu_chart = BarChart::new("Overall CPU", cpu_overall, 40);
            cpu_chart.draw(&mut stdout)?;
            
            execute!(stdout, Print("\n"))?;
            
            // Draw individual core bar charts
            for (i, usage) in cpu_per_core.iter().enumerate() {
                let core_chart = BarChart::new(&format!("Core #{}", i), *usage, 40);
                core_chart.draw(&mut stdout)?;
            }
            
            // Display Memory information
            execute!(
                stdout,
                Print("\n"),
                SetForegroundColor(Color::Cyan),
                Print("=== MEMORY USAGE ===\n"),
                ResetColor,
                Print(format!(
                    "Total Memory: {} MB\n", total_mem
                )),
                Print(format!(
                    "Used Memory:  {} MB\n", used_mem
                ))
            )?;
            
            // Draw memory usage bar chart
            let mem_chart = BarChart::new("Memory", mem_usage, 40);
            mem_chart.draw(&mut stdout)?;
            
            // Display NVIDIA GPU information if available
            #[cfg(feature = "nvidia-gpu")]
            {
                if let Some(_) = &self.gpu_monitor {
                    let gpu_info = self.get_gpu_info();
                    
                    if !gpu_info.is_empty() {
                        execute!(
                            stdout,
                            Print("\n"),
                            SetForegroundColor(Color::Cyan),
                            Print("=== NVIDIA GPU USAGE ===\n"),
                            ResetColor
                        )?;
                        
                        for (i, gpu) in gpu_info.iter().enumerate() {
                            execute!(
                                stdout,
                                Print(format!("GPU #{}: {}\n", i, gpu.name)),
                                Print(format!("Temperature: {}°C\n", gpu.temperature))
                            )?;
                            
                            // Draw GPU utilization bar chart
                            let gpu_util_chart = BarChart::new("GPU Utilization", gpu.utilization, 40);
                            gpu_util_chart.draw(&mut stdout)?;
                            
                            // Draw GPU memory usage bar chart
                            let gpu_mem_chart = BarChart::new("GPU Memory", gpu.memory_usage, 40);
                            gpu_mem_chart.draw(&mut stdout)?;
                            
                            execute!(stdout, Print("\n"))?;
                        }
                    }
                }
            }
            
            // Display Apple GPU information if available
            #[cfg(feature = "apple-gpu")]
            {
                if let Some(_) = &self.mac_gpu_monitor {
                    let gpu_info = self.get_mac_gpu_info();
                    
                    if !gpu_info.is_empty() {
                        execute!(
                            stdout,
                            Print("\n"),
                            SetForegroundColor(Color::Cyan),
                            Print("=== APPLE GPU USAGE ===\n"),
                            ResetColor
                        )?;
                        
                        for (i, gpu) in gpu_info.iter().enumerate() {
                            execute!(
                                stdout,
                                Print(format!("GPU #{}: {}\n", i, gpu.name)),
                                Print(format!("Type: {}\n", 
                                    if gpu.is_headless { "Headless" } 
                                    else if gpu.is_low_power { "Integrated/Low Power" } 
                                    else { "Discrete/High Performance" }
                                ))
                            )?;
                            
                            // Draw GPU utilization bar chart (estimated)
                            let gpu_util_chart = BarChart::new("GPU Utilization (est.)", gpu.utilization, 40);
                            gpu_util_chart.draw(&mut stdout)?;
                            
                            // Show total memory
                            execute!(
                                stdout,
                                Print(format!("Total Memory: {} MB\n\n", gpu.total_memory))
                            )?;
                        }
                    }
                }
            }
            
            // Wait for next refresh
            thread::sleep(self.refresh_rate);
        }
    }
}

fn main() -> io::Result<()> {
    // Create system monitor with 1000ms (1 second) refresh rate
    let mut monitor = SystemMonitor::new(1000);
    
    // Run the display loop
    monitor.display()
}