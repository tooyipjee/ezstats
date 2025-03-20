// ezstats - A lightweight system monitoring tool
// A terminal-based system monitor with interactive UI for displaying
// real-time CPU, RAM, and GPU usage statistics

use sysinfo::{System, SystemExt, CpuExt};
use std::{io, thread, time::Duration};
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{MoveTo, Hide, Show},
    style::{Color, Print, ResetColor, SetForegroundColor},
    event::{self, Event, KeyCode, KeyModifiers},
};

mod gpu;
mod widget;
mod ui;

use gpu::{GpuMonitor, GpuInfo, GpuVendor};
use widget::{Widget, BarChart};
use ui::{UiState, ViewType};

// Simplified system monitor with unified GPU detection
struct SystemMonitor {
    system: System,
    refresh_rate: Duration,
    gpu_monitor: GpuMonitor,
}

impl SystemMonitor {
    /// Create a new SystemMonitor with the given refresh rate in milliseconds
    fn new(refresh_ms: u64) -> Self {
        // Initialize system information
        let mut system = System::new_all();
        system.refresh_all();
        
        // Initialize GPU monitoring with automatic detection
        let gpu_monitor = GpuMonitor::new();
        
        // Log detected hardware
        println!("Detected {} CPU cores", system.cpus().len());
        println!("Detected {} GPUs", gpu_monitor.device_count());
        
        SystemMonitor {
            system,
            refresh_rate: Duration::from_millis(refresh_ms),
            gpu_monitor,
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
            
        let overall_usage = if per_cpu.is_empty() {
            0.0
        } else {
            per_cpu.iter().sum::<f32>() / per_cpu.len() as f32
        };
        
        (per_cpu, overall_usage)
    }
    
    /// Get memory information in MB
    fn get_memory_info(&self) -> (u64, u64, f32) {
        let total_mem = self.system.total_memory() / 1024 / 1024; // Convert to MB
        let used_mem = self.system.used_memory() / 1024 / 1024;   // Convert to MB
        let mem_usage_pct = if total_mem > 0 {
            (used_mem as f32 / total_mem as f32) * 100.0
        } else {
            0.0
        };
        
        (total_mem, used_mem, mem_usage_pct)
    }
    
    /// Get GPU information
    fn get_gpu_info(&self) -> Vec<GpuInfo> {
        self.gpu_monitor.get_gpu_info()
    }
    
    /// Check if there are any GPUs available
    fn has_gpus(&self) -> bool {
        self.gpu_monitor.has_gpus()
    }
    
    /// Run the interactive display loop
    fn display(&mut self) -> io::Result<()> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        
        // Create UI state
        let mut ui_state = UiState::new(self.has_gpus());
        
        // Process events and update display
        let result = self.run_event_loop(&mut stdout, &mut ui_state);
        
        // Clean up terminal before returning
        execute!(stdout, Show, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        
        // Propagate any errors from the event loop
        result
    }
    
    /// Main event loop - handles keyboard events and updates display
    fn run_event_loop<W: io::Write>(&mut self, stdout: &mut W, ui_state: &mut UiState) -> io::Result<()> {
        while ui_state.running {
            // Check if we need to update system data
            let needs_update = ui_state.should_update(self.refresh_rate);
            
            // Use a shorter polling timeout to improve responsiveness
            if crossterm::event::poll(Duration::from_millis(50))? {
                if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                    // Process key event - returns true if UI needs updating
                    let ui_changed = ui::handle_key_event(key_event, ui_state);
                    
                    // If the quit key was pressed, exit the loop immediately
                    if !ui_state.running {
                        break;
                    }
                    
                    // If UI changed, force an update
                    if ui_changed {
                        self.render_current_view(stdout, ui_state)?;
                    }
                }
            }
            
            // Update system data if needed
            if needs_update && ui_state.automatic_refresh {
                self.refresh();
                ui_state.mark_updated();
                
                // Render current view
                self.render_current_view(stdout, ui_state)?;
            }
        }
        
        Ok(())
    }
    
    /// Render the current view based on UI state
    fn render_current_view<W: io::Write>(&self, stdout: &mut W, ui_state: &UiState) -> io::Result<()> {
        // Draw common UI frame
        ui::draw_ui_frame(stdout, ui_state)?;
        
        // Get current system metrics
        let (cpu_per_core, cpu_overall) = self.get_cpu_usage();
        let (total_mem, used_mem, mem_usage) = self.get_memory_info();
        
        // Get GPU data
        let gpu_info = self.get_gpu_info();
        
        // Draw the appropriate view based on current state
        match ui_state.views.current() {
            ViewType::Overview => {
                ui::draw_overview_view(
                    stdout, 
                    cpu_overall, 
                    mem_usage,
                    &gpu_info,
                )?;
            },
            ViewType::CpuDetailed => {
                ui::draw_cpu_view(stdout, cpu_overall, &cpu_per_core)?;
            },
            ViewType::MemoryDetailed => {
                ui::draw_memory_view(stdout, total_mem, used_mem, mem_usage)?;
            },
            ViewType::GpuDetailed => {
                if self.has_gpus() {
                    ui::draw_gpu_view(stdout, &gpu_info)?;
                } else {
                    ui::draw_no_gpu_view(stdout)?;
                }
            },
            ViewType::Help => {
                ui::draw_help_view(stdout)?;
            },
        }
        
        stdout.flush()?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // Handle unexpected errors gracefully
    match run_app() {
        Ok(_) => Ok(()),
        Err(e) => {
            // Make sure we restore terminal state on error
            if let Err(term_err) = cleanup_terminal() {
                eprintln!("Failed to clean up terminal: {}", term_err);
            }
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}

// The actual application logic
fn run_app() -> io::Result<()> {
    // Create system monitor with 1000ms (1 second) refresh rate
    let mut monitor = SystemMonitor::new(1000);
    
    // Run the interactive display loop
    monitor.display()
}

// Clean up the terminal state in case of error
fn cleanup_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()
}