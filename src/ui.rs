// src/ui.rs
//
// Interactive UI system with views and keyboard navigation
// Simplified to remove feature flags and unify GPU display

use std::io::{self, Write};
use std::time::{Duration, Instant};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    execute,
};

use crate::widget::Widget;
use crate::gpu::{GpuInfo, GpuVendor};
use crate::widget::BarChart;

// View types that can be displayed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    Overview,
    CpuDetailed,
    MemoryDetailed,
    GpuDetailed,
    Help,
}

impl ViewType {
    // Get a user-friendly name for the view
    pub fn name(&self) -> &'static str {
        match self {
            ViewType::Overview => "Overview",
            ViewType::CpuDetailed => "CPU Details",
            ViewType::MemoryDetailed => "Memory Details",
            ViewType::GpuDetailed => "GPU Details",
            ViewType::Help => "Help",
        }
    }
}

// Available views for navigation
pub struct Views {
    current: ViewType,
    available: Vec<ViewType>,
}

impl Views {
    pub fn new(has_gpu: bool) -> Self {
        let mut available = vec![
            ViewType::Overview,
            ViewType::CpuDetailed,
            ViewType::MemoryDetailed,
        ];
        
        // Add GPU view if we have a GPU
        if has_gpu {
            available.push(ViewType::GpuDetailed);
        }
        
        available.push(ViewType::Help);
        
        Views {
            current: ViewType::Overview,
            available,
        }
    }
    
    pub fn current(&self) -> ViewType {
        self.current
    }
    
    pub fn next(&mut self) {
        if let Some(index) = self.available.iter().position(|&v| v == self.current) {
            let next_index = (index + 1) % self.available.len();
            self.current = self.available[next_index];
        }
    }
    
    pub fn prev(&mut self) {
        if let Some(index) = self.available.iter().position(|&v| v == self.current) {
            let next_index = if index == 0 {
                self.available.len() - 1
            } else {
                index - 1
            };
            self.current = self.available[next_index];
        }
    }
    
    pub fn go_to(&mut self, view_type: ViewType) {
        if self.available.contains(&view_type) {
            self.current = view_type;
        }
    }
}

// State data shared between views
pub struct UiState {
    pub views: Views,
    pub running: bool,
    pub automatic_refresh: bool,
    pub last_update: Instant,
    pub show_help_line: bool,
}

impl UiState {
    pub fn new(has_gpu: bool) -> Self {
        UiState {
            views: Views::new(has_gpu),
            running: true,
            automatic_refresh: true,
            last_update: Instant::now(),
            show_help_line: true,
        }
    }
    
    pub fn toggle_automatic_refresh(&mut self) {
        self.automatic_refresh = !self.automatic_refresh;
    }
    
    pub fn should_update(&self, refresh_rate: Duration) -> bool {
        self.automatic_refresh && self.last_update.elapsed() >= refresh_rate
    }
    
    pub fn mark_updated(&mut self) {
        self.last_update = Instant::now();
    }
}

// Process keyboard events and update UI state accordingly
pub fn handle_key_event(key_event: KeyEvent, state: &mut UiState) -> bool {
    // Handle quit keys first for immediate response
    if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc) || 
       (matches!(key_event.code, KeyCode::Char('c')) && key_event.modifiers.contains(KeyModifiers::CONTROL)) {
        // Set the running flag to false to exit the main loop
        state.running = false;
        return true; // UI changed
    }
    
    match key_event.code {
        // Navigation
        KeyCode::Tab => state.views.next(),
        KeyCode::BackTab => state.views.prev(),
        KeyCode::Char('1') => state.views.go_to(ViewType::Overview),
        KeyCode::Char('2') => state.views.go_to(ViewType::CpuDetailed),
        KeyCode::Char('3') => state.views.go_to(ViewType::MemoryDetailed),
        KeyCode::Char('4') => state.views.go_to(ViewType::GpuDetailed),
        KeyCode::Char('?') | KeyCode::Char('h') => state.views.go_to(ViewType::Help),
        
        // Controls
        KeyCode::Char('p') => state.toggle_automatic_refresh(),
        KeyCode::Char('r') => {
            state.mark_updated();
            return true; // Force refresh
        },
        
        _ => return false, // No UI change needed
    }
    
    true // UI changed, need to redraw
}

// Helper function to draw a content box with a title
pub fn draw_content_box<W: Write>(
    stdout: &mut W, 
    title: &str, 
    start_row: u16,
    height: u16
) -> io::Result<()> {
    let (term_width, _) = match crossterm::terminal::size() {
        Ok((w, h)) => (w, h),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Top border with title
    execute!(
        stdout,
        MoveTo(0, start_row),
        SetForegroundColor(Color::Blue),
        Print("┌"),
        Print("─".repeat((term_width - 2) as usize)),
        Print("┐"),
        ResetColor
    )?;
    
    // Add title to the top border
    execute!(
        stdout,
        MoveTo(2, start_row),
        SetForegroundColor(Color::Cyan),
        Print(format!(" {} ", title)),
        ResetColor
    )?;
    
    // Draw side borders
    for y in (start_row + 1)..height {
        execute!(
            stdout,
            MoveTo(0, y),
            SetForegroundColor(Color::Blue),
            Print("│"),
            MoveTo(term_width - 1, y),
            Print("│"),
            ResetColor
        )?;
    }
    
    // Bottom border
    execute!(
        stdout,
        MoveTo(0, height),
        SetForegroundColor(Color::Blue),
        Print("└"),
        Print("─".repeat((term_width - 2) as usize)),
        Print("┘"),
        ResetColor
    )?;
    
    Ok(())
}

// Draw common UI elements like titlebar and help line
pub fn draw_ui_frame<W: Write>(stdout: &mut W, state: &UiState) -> io::Result<()> {
    let view_name = state.views.current().name();
    
    // Get terminal dimensions
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w, h),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Clear the screen
    execute!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    
    // Title bar
    execute!(
        stdout,
        SetForegroundColor(Color::Blue),
        Print("═".repeat(term_width as usize)),
        ResetColor
    )?;
    
    // App name
    execute!(
        stdout,
        MoveTo(2, 0),
        SetForegroundColor(Color::White),
        Print(" ezstats "),
        ResetColor
    )?;
    
    // View title - centered
    let view_title = format!(" {} ", view_name);
    let center_pos = (term_width as usize - view_title.len()) / 2;
    execute!(
        stdout,
        MoveTo(center_pos as u16, 0),
        SetForegroundColor(Color::White),
        Print(view_title),
        ResetColor
    )?;
    
    // Status indicator (frozen/active)
    let status = if state.automatic_refresh {
        " ACTIVE "
    } else {
        " PAUSED "
    };
    
    let status_color = if state.automatic_refresh {
        Color::Green
    } else {
        Color::Yellow
    };
    
    execute!(
        stdout,
        MoveTo(term_width - status.len() as u16 - 2, 0),
        SetForegroundColor(status_color),
        Print(status),
        ResetColor
    )?;
    
    // Help line at the bottom
    if state.show_help_line {
        let help_text = " [?] Help | [Tab] Next view | [1-4] Switch view | [p] Pause/resume | [r] Refresh | [q] Quit ";
        
        execute!(
            stdout,
            MoveTo(0, term_height - 1),
            SetForegroundColor(Color::DarkGrey),
            Print(help_text),
            ResetColor
        )?;
    }
    
    Ok(())
}

// Draw the overview view - a simplified version of all metrics
pub fn draw_overview_view<W: Write>(
    stdout: &mut W, 
    cpu_usage: f32,
    memory_usage: f32,
    gpu_info: &[GpuInfo],
) -> io::Result<()> {
    // Get terminal dimensions to properly size content
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Calculate content box dimensions (leaving room for borders)
    let content_width = term_width.saturating_sub(4);
    let bar_width = content_width.saturating_sub(25); // Allow space for labels and values
    
    // Create a content area with a border
    draw_content_box(stdout, "System Overview", 2, term_height as u16 - 3)?;
    
    // Start content 1 row below the header, 2 columns in from the left
    let content_start_x = 2;
    let content_start_y = 3;
    let mut current_row = content_start_y;
    
    // Draw CPU usage
    execute!(stdout, MoveTo(content_start_x, current_row))?;
    let cpu_chart = BarChart::new("CPU Usage", cpu_usage, bar_width);
    cpu_chart.draw(stdout)?;
    current_row += 2;
    
    // Draw memory usage
    execute!(stdout, MoveTo(content_start_x, current_row))?;
    let mem_chart = BarChart::new("Memory Usage", memory_usage, bar_width);
    mem_chart.draw(stdout)?;
    current_row += 2;
    
    // Draw GPU usage if available
    if !gpu_info.is_empty() {
        for (i, gpu) in gpu_info.iter().enumerate().take(1) { // Just show the first GPU in overview
            execute!(stdout, MoveTo(content_start_x, current_row))?;
            let gpu_usage_chart = BarChart::new(&format!("GPU #{} Usage", i), gpu.utilization, bar_width);
            gpu_usage_chart.draw(stdout)?;
            current_row += 1;
            
            execute!(stdout, MoveTo(content_start_x, current_row))?;
            let gpu_mem_chart = BarChart::new(&format!("GPU #{} Memory", i), gpu.memory_usage, bar_width);
            gpu_mem_chart.draw(stdout)?;
            current_row += 2;
        }
    }
    
    Ok(())
}

// Draw CPU-specific view with detailed information
pub fn draw_cpu_view<W: Write>(stdout: &mut W, cpu_overall: f32, cpu_per_core: &[f32]) -> io::Result<()> {
    // Get terminal dimensions to properly size content
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Calculate content box dimensions
    let content_width = term_width.saturating_sub(4);
    let bar_width = content_width.saturating_sub(25); // Allow space for labels and values
    
    // Create a content area with a border
    draw_content_box(stdout, "CPU Details", 2, term_height as u16 - 3)?;
    
    // Start content 1 row below the header, 2 columns in from the left
    let content_start_x = 2;
    let content_start_y = 3;
    let mut current_row = content_start_y;
    
    // Draw overall CPU usage
    execute!(stdout, MoveTo(content_start_x, current_row))?;
    let cpu_chart = BarChart::new("Overall CPU", cpu_overall, bar_width);
    cpu_chart.draw(stdout)?;
    current_row += 2; // Add some spacing
    
    // Draw individual core bar charts
    for (i, usage) in cpu_per_core.iter().enumerate() {
        execute!(stdout, MoveTo(content_start_x, current_row))?;
        let core_chart = BarChart::new(&format!("Core #{}", i), *usage, bar_width);
        core_chart.draw(stdout)?;
        current_row += 1; // Each core on its own row
    }
    
    Ok(())
}

// Draw memory-specific view
pub fn draw_memory_view<W: Write>(
    stdout: &mut W, 
    total_mem: u64, 
    used_mem: u64, 
    mem_usage: f32
) -> io::Result<()> {
    // Get terminal dimensions to properly size content
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Calculate content box dimensions
    let content_width = term_width.saturating_sub(4);
    let bar_width = content_width.saturating_sub(25); // Allow space for labels and values
    
    // Create a content area with a border
    draw_content_box(stdout, "Memory Details", 2, term_height as u16 - 3)?;
    
    // Start content 1 row below the header, 2 columns in from the left
    let content_start_x = 2;
    let content_start_y = 3;
    let mut current_row = content_start_y;
    
    // Memory statistics in a neatly formatted table
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        SetForegroundColor(Color::White),
        Print("Memory Statistics:"),
        ResetColor
    )?;
    current_row += 1;
    
    // Format memory values with consistent alignment
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        Print(format!("┌{:─^40}┐", "")),
    )?;
    current_row += 1;
    
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        Print(format!("│ {:20} │ {:16} │", "Total Memory:", format!("{} MB", total_mem))),
    )?;
    current_row += 1;
    
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        Print(format!("│ {:20} │ {:16} │", "Used Memory:", format!("{} MB", used_mem))),
    )?;
    current_row += 1;
    
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        Print(format!("│ {:20} │ {:16} │", "Free Memory:", format!("{} MB", total_mem - used_mem))),
    )?;
    current_row += 1;
    
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        Print(format!("│ {:20} │ {:16} │", "Usage Percentage:", format!("{:.1}%", mem_usage))),
    )?;
    current_row += 1;
    
    execute!(
        stdout,
        MoveTo(content_start_x, current_row),
        Print(format!("└{:─^40}┘", "")),
    )?;
    current_row += 2;
    
    // Draw memory usage bar chart
    execute!(stdout, MoveTo(content_start_x, current_row))?;
    let mem_chart = BarChart::new("Memory Usage", mem_usage, bar_width);
    mem_chart.draw(stdout)?;
    
    Ok(())
}

// Draw GPU-specific view - unified for all GPU types
pub fn draw_gpu_view<W: Write>(
    stdout: &mut W,
    gpu_info: &[GpuInfo],
) -> io::Result<()> {
    // Get terminal dimensions to properly size content
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Calculate content box dimensions
    let content_width = term_width.saturating_sub(4);
    let bar_width = content_width.saturating_sub(25); // Allow space for labels and values
    
    // Create a content area with a border
    draw_content_box(stdout, "GPU Details", 2, term_height as u16 - 3)?;
    
    // Start content 1 row below the header, 2 columns in from the left
    let content_start_x = 2;
    let content_start_y = 3;
    let mut current_row = content_start_y;
    
    if gpu_info.is_empty() {
        // Show a message if no GPUs are available
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            Print("No GPU monitoring available."),
            MoveTo(content_start_x, current_row + 1),
            Print("No compatible GPUs detected on your system.")
        )?;
        return Ok(());
    }
    
            // Display GPU information
    for (i, gpu) in gpu_info.iter().enumerate() {
        // GPU vendor label
        let vendor_label = match gpu.vendor {
            GpuVendor::Nvidia => "NVIDIA GPU",
            GpuVendor::Apple => "Apple GPU",
            GpuVendor::Other => "GPU",
            GpuVendor::None => "Unknown GPU",
        };
        
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            SetForegroundColor(Color::Green),
            Print(format!("=== {} #{} ===", vendor_label, i)),
            ResetColor
        )?;
        current_row += 2;
        
        // GPU info table
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            Print(format!("┌{:─^40}┐", "")),
        )?;
        current_row += 1;
        
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            Print(format!("│ {:^40} │", gpu.name)),
        )?;
        current_row += 1;
        
        // Only show temperature for NVIDIA GPUs
        if gpu.vendor == GpuVendor::Nvidia {
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                Print(format!("│ {:20} │ {:16} │", "Temperature:", format!("{}°C", gpu.temperature))),
            )?;
            current_row += 1;
        }
        
        // Show Apple-specific properties for Apple GPUs
        if gpu.vendor == GpuVendor::Apple {
            let gpu_type = if gpu.is_headless { 
                "Headless" 
            } else if gpu.is_low_power { 
                "Integrated/Low Power" 
            } else { 
                "Discrete/High Performance" 
            };
            
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                Print(format!("│ {:20} │ {:16} │", "Type:", gpu_type)),
            )?;
            current_row += 1;
        }
        
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            Print(format!("│ {:20} │ {:16} │", "Memory:", format!("{} MB", gpu.total_memory))),
        )?;
        current_row += 1;
        
        if gpu.vendor == GpuVendor::Nvidia {
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                Print(format!("│ {:20} │ {:16} │", "Memory Usage:", format!("{} / {} MB", gpu.used_memory, gpu.total_memory))),
            )?;
            current_row += 1;
        }
        
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            Print(format!("└{:─^40}┘", "")),
        )?;
        current_row += 2;
        
        // Draw GPU utilization bar chart
        execute!(stdout, MoveTo(content_start_x, current_row))?;
        let gpu_util_chart = BarChart::new("GPU Utilization", gpu.utilization, bar_width);
        gpu_util_chart.draw(stdout)?;
        current_row += 1;
        
        // Draw GPU memory usage bar chart for NVIDIA
        if gpu.vendor == GpuVendor::Nvidia {
            execute!(stdout, MoveTo(content_start_x, current_row))?;
            let gpu_mem_chart = BarChart::new("GPU Memory", gpu.memory_usage, bar_width);
            gpu_mem_chart.draw(stdout)?;
        }
        
        current_row += 2;
    }
    
    Ok(())
}

// Draw view for when no GPU is available
pub fn draw_no_gpu_view<W: Write>(stdout: &mut W) -> io::Result<()> {
    // Get terminal dimensions to properly size content
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Create a content area with a border
    draw_content_box(stdout, "GPU Details", 2, term_height as u16 - 3)?;
    
    // Start content 1 row below the header, 2 columns in from the left
    let content_start_x = 2;
    let content_start_y = 3;
    
    // Show a message that no GPU is available
    execute!(
        stdout,
        MoveTo(content_start_x, content_start_y),
        Print("No compatible GPUs detected on your system."),
        MoveTo(content_start_x, content_start_y + 2),
        Print("ezstats currently supports:"),
        MoveTo(content_start_x, content_start_y + 3),
        Print("  - NVIDIA GPUs with appropriate drivers installed"),
        MoveTo(content_start_x, content_start_y + 4),
        Print("  - Apple Silicon M-series and Intel Macs with Metal support")
    )?;
    
    Ok(())
}

// Draw help view with keyboard shortcuts
pub fn draw_help_view<W: Write>(stdout: &mut W) -> io::Result<()> {
    // Get terminal dimensions to properly size content
    let (term_width, term_height) = match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24), // Fallback to a reasonable default
    };
    
    // Create a content area with a border
    draw_content_box(stdout, "Keyboard Controls", 2, term_height as u16 - 3)?;
    
    // Start content 1 row below the header, 2 columns in from the left
    let content_start_x = 4; // Indent a bit more for better readability
    let content_start_y = 3;
    let mut current_row = content_start_y;
    
    let help_items = [
        ("Navigation", ""),
        ("  Tab", "Next view"),
        ("  Shift+Tab", "Previous view"),
        ("  1", "Overview"),
        ("  2", "CPU details"),
        ("  3", "Memory details"),
        ("  4", "GPU details (if available)"),
        ("  ? or h", "Show this help"),
        ("", ""),
        ("Controls", ""),
        ("  p", "Pause/resume automatic updates"),
        ("  r", "Force refresh now"),
        ("", ""),
        ("Exit", ""),
        ("  q or Esc", "Quit"),
        ("  Ctrl+c", "Quit"),
    ];
    
    for (key, description) in help_items.iter() {
        if key.is_empty() {
            current_row += 1; // Add empty row for spacing
        } else if !description.is_empty() {
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                SetForegroundColor(Color::Yellow),
                Print(format!("{:12}", key)),
                ResetColor,
                Print(format!(" → {}", description))
            )?;
            current_row += 1;
        } else {
            // Section header
            execute!(
                stdout,
                MoveTo(content_start_x - 2, current_row),
                SetForegroundColor(Color::Green),
                Print(format!("» {}", key)),
                ResetColor
            )?;
            current_row += 1;
        }
    }
    
    // Add a note about the application at the bottom
    execute!(
        stdout,
        MoveTo(content_start_x, term_height as u16 - 5),
        SetForegroundColor(Color::DarkGrey),
        Print("ezstats is a lightweight terminal-based system monitor"),
        MoveTo(content_start_x, term_height as u16 - 4),
        Print("designed for minimal resource usage while providing"),
        MoveTo(content_start_x, term_height as u16 - 3),
        Print("real-time monitoring of system resources."),
        ResetColor
    )?;
    
    Ok(())
}