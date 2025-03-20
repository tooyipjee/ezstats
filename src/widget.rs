// src/widget.rs
// Improved widgets with better error handling and rendering

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
        // Ensure width is reasonable
        let safe_width = width.max(10).min(200);
        
        BarChart {
            title: title.to_string(),
            value: value.clamp(0.0, 100.0),
            width: safe_width,
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
    
    /// Get a textual representation of the value for display
    fn format_value(&self) -> String {
        format!("{:.1}%", self.value)
    }
}

impl Widget for BarChart {
    fn draw(&self, stdout: &mut impl Write) -> io::Result<()> {
        // Calculate the filled portion of the bar
        let filled_width = ((self.value / 100.0) * self.width as f32).round() as usize;
        let empty_width = self.width.saturating_sub(filled_width);
        
        // Fixed title column width for alignment (align all chart titles consistently)
        const TITLE_COLUMN_WIDTH: usize = 15;
        
        // Format title with fixed width for alignment
        let title_display = format!("{:<width$}", self.title, width = TITLE_COLUMN_WIDTH);
        
        // Draw the title with consistent alignment
        execute!(
            stdout,
            SetForegroundColor(Color::White),
            crossterm::style::Print(title_display),
            ResetColor
        )?;
        
        // Draw the filled portion (if any)
        if filled_width > 0 {
            execute!(
                stdout,
                SetForegroundColor(self.get_color()),
                crossterm::style::Print("█".repeat(filled_width)),
                ResetColor
            )?;
        }
        
        // Draw the empty portion (if any)
        if empty_width > 0 {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                crossterm::style::Print("░".repeat(empty_width)),
                ResetColor
            )?;
        }
        
        // Fixed value width for consistent display
        let value_text = format!(" {:>6}", self.format_value());
        
        // Draw the percentage value with consistent alignment
        execute!(
            stdout,
            SetForegroundColor(self.get_color()),
            crossterm::style::Print(value_text),
            ResetColor,
            crossterm::style::Print("\n")
        )?;
        
        Ok(())
    }
}

/// A simple text widget for displaying information
pub struct TextWidget {
    lines: Vec<String>,
    color: Option<Color>,
}

impl TextWidget {
    /// Create a new text widget with the given lines
    pub fn new(text: &str) -> Self {
        let lines = text.split('\n').map(|s| s.to_string()).collect();
        
        TextWidget {
            lines,
            color: None,
        }
    }
    
    /// Set the text color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for TextWidget {
    fn draw(&self, stdout: &mut impl Write) -> io::Result<()> {
        // Apply color if specified
        if let Some(color) = self.color {
            execute!(stdout, SetForegroundColor(color))?;
        }
        
        // Draw each line
        for line in &self.lines {
            execute!(stdout, crossterm::style::Print(line), crossterm::style::Print("\n"))?;
        }
        
        // Reset color if we set one
        if self.color.is_some() {
            execute!(stdout, ResetColor)?;
        }
        
        Ok(())
    }
}