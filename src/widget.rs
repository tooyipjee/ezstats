// src/widget.rs
use std::io::{self, Write};
use crossterm::{
    style::{Color, SetForegroundColor, ResetColor},
    execute,
};

/// Represents a simple widget that can be drawn in the terminal
pub trait Widget {
    fn draw(&self, stdout: &mut impl Write) -> io::Result<()>;
}

/// A bar chart widget for displaying usage metrics (CPU, RAM)
pub struct BarChart {
    title: String,
    value: f32,  // Value as a percentage (0-100)
    width: usize,
}

impl BarChart {
    /// Create a new bar chart with the given title and value
    pub fn new(title: &str, value: f32, width: usize) -> Self {
        BarChart {
            title: title.to_string(),
            value: value.clamp(0.0, 100.0),
            width,
        }
    }
    
    /// Get the appropriate color based on the value
    fn get_color(&self) -> Color {
        if self.value > 80.0 {
            Color::Red
        } else if self.value > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }
}

impl Widget for BarChart {
    fn draw(&self, stdout: &mut impl Write) -> io::Result<()> {
        // Calculate the filled portion of the bar
        let filled_width = ((self.value / 100.0) * self.width as f32).round() as usize;
        let empty_width = self.width - filled_width;
        
        // Draw the title
        execute!(
            stdout,
            SetForegroundColor(Color::White),
            crossterm::style::Print(format!("{:<15}", self.title)),
            ResetColor
        )?;
        
        // Draw the filled portion
        execute!(
            stdout,
            SetForegroundColor(self.get_color()),
            crossterm::style::Print("█".repeat(filled_width)),
            ResetColor
        )?;
        
        // Draw the empty portion
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            crossterm::style::Print("░".repeat(empty_width)),
            ResetColor
        )?;
        
        // Draw the percentage value
        execute!(
            stdout,
            SetForegroundColor(self.get_color()),
            crossterm::style::Print(format!(" {:.1}%", self.value)),
            ResetColor,
            crossterm::style::Print("\n")
        )?;
        
        Ok(())
    }
}