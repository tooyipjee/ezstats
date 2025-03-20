// src/ui.rs
//
// Interactive UI system with views and keyboard navigation
// Inspired by TUI applications like lazygit

use std::io::{self, Write};
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    execute,
};

use crate::widget::Widget;
use crate::gpu::GpuMonitor;
#[cfg(feature = "nvidia-gpu")]
use crate::gpu::GpuInfo;
#[cfg(feature = "apple-gpu")]
use crate::mac_gpu::{MacGpuMonitor, MacGpuInfo};
use crate::widget::BarChart;

// View types that can be displayed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    Overview,
    CpuDetailed,
    MemoryDetailed,
    #[cfg(any(feature = "nvidia-gpu", feature = "apple-gpu"))]
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
            #[cfg(any(feature = "nvidia-gpu", feature = "apple-gpu"))]
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
    pub fn new() -> Self {
        let mut available = vec![
            ViewType::Overview,
            ViewType::CpuDetailed,
            ViewType::MemoryDetailed,
        ];
        
        #[cfg(any(feature = "nvidia-gpu", feature = "apple-gpu"))]
        available.push(ViewType::GpuDetailed);
        
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
    pub fn new() -> Self {
        UiState {
            views: Views::new(),
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
        #[cfg(any(feature = "nvidia-gpu", feature = "apple-gpu"))]
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
    #[cfg(feature = "nvidia-gpu")]
    gpu_info: &[GpuInfo],
    #[cfg(feature = "apple-gpu")]
    mac_gpu_info: &[MacGpuInfo],
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
    #[cfg(feature = "nvidia-gpu")]
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
    
    #[cfg(feature = "apple-gpu")]
    if !mac_gpu_info.is_empty() {
        for (i, gpu) in mac_gpu_info.iter().enumerate().take(1) { // Just show the first GPU in overview
            execute!(stdout, MoveTo(content_start_x, current_row))?;
            let gpu_usage_chart = BarChart::new(&format!("GPU #{} Usage", i), gpu.utilization, bar_width);
            gpu_usage_chart.draw(stdout)?;
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

// Draw GPU-specific view
#[cfg(any(feature = "nvidia-gpu", feature = "apple-gpu"))]
pub fn draw_gpu_view<W: Write>(
    stdout: &mut W,
    #[cfg(feature = "nvidia-gpu")]
    gpu_info: &[GpuInfo],
    #[cfg(feature = "apple-gpu")]
    mac_gpu_info: &[MacGpuInfo],
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
    
    let has_gpu_info = 
        #[cfg(feature = "nvidia-gpu")]
        !gpu_info.is_empty() ||
        #[cfg(feature = "apple-gpu")]
        !mac_gpu_info.is_empty();
    
    #[cfg(not(any(feature = "nvidia-gpu", feature = "apple-gpu")))]
    let has_gpu_info = false;
    
    if !has_gpu_info {
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            Print("No GPU monitoring available.\n"),
            MoveTo(content_start_x, current_row + 1),
            Print("Rebuild with --features nvidia-gpu or --features apple-gpu to enable.")
        )?;
        return Ok(());
    }
    
    // Display NVIDIA GPU information if available
    #[cfg(feature = "nvidia-gpu")]
    if !gpu_info.is_empty() {
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            SetForegroundColor(Color::Green),
            Print("=== NVIDIA GPUs ==="),
            ResetColor
        )?;
        current_row += 2;
        
        for (i, gpu) in gpu_info.iter().enumerate() {
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
                Print(format!("│ {:^40} │", format!("GPU #{}: {}", i, gpu.name))),
            )?;
            current_row += 1;
            
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                Print(format!("│ {:20} │ {:16} │", "Temperature:", format!("{}°C", gpu.temperature))),
            )?;
            current_row += 1;
            
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                Print(format!("│ {:20} │ {:16} │", "Memory Usage:", format!("{} / {} MB", gpu.used_memory, gpu.total_memory))),
            )?;
            current_row += 1;
            
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
            
            // Draw GPU memory usage bar chart
            execute!(stdout, MoveTo(content_start_x, current_row))?;
            let gpu_mem_chart = BarChart::new("GPU Memory", gpu.memory_usage, bar_width);
            gpu_mem_chart.draw(stdout)?;
            current_row += 2;
        }
    }
    
    // Display Apple GPU information if available
    #[cfg(feature = "apple-gpu")]
    if !mac_gpu_info.is_empty() {
        execute!(
            stdout,
            MoveTo(content_start_x, current_row),
            SetForegroundColor(Color::Green),
            Print("=== Apple GPUs ==="),
            ResetColor
        )?;
        current_row += 2;
        
        for (i, gpu) in mac_gpu_info.iter().enumerate() {
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
                Print(format!("│ {:^40} │", format!("GPU #{}: {}", i, gpu.name))),
            )?;
            current_row += 1;
            
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
            
            execute!(
                stdout,
                MoveTo(content_start_x, current_row),
                Print(format!("│ {:20} │ {:16} │", "Total Memory:", format!("{} MB", gpu.total_memory))),
            )?;
            current_row += 1;
            
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
            current_row += 2;
        }
    }
    
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
        #[cfg(any(feature = "nvidia-gpu", feature = "apple-gpu"))]
        ("  4", "GPU details"),
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