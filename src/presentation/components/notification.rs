use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Error,
    Warning,
    Success,
    Info,
}

impl NotificationType {
    pub fn color(&self) -> Color {
        match self {
            NotificationType::Error => Color::Red,
            NotificationType::Warning => Color::Yellow,
            NotificationType::Success => Color::Green,
            NotificationType::Info => Color::Blue,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    created_at: Instant,
    duration: Duration,
}

impl Notification {
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    pub fn error(message: String) -> Self {
        Self::new(message, NotificationType::Error)
    }

    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    pub fn success(message: String) -> Self {
        Self::new(message, NotificationType::Success)
    }

    pub fn info(message: String) -> Self {
        Self::new(message, NotificationType::Info)
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    fn get_animation_progress(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_millis() as f32;
        let total = self.duration.as_millis() as f32;
        
        // Fade in durante los primeros 200ms, fade out durante los últimos 500ms
        if elapsed < 200.0 {
            elapsed / 200.0 // Fade in
        } else if elapsed > total - 500.0 {
            (total - elapsed) / 500.0 // Fade out
        } else {
            1.0 // Completamente visible
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        if self.is_expired() {
            return;
        }

        let progress = self.get_animation_progress();
        let notification_width = self.message.len() as u16 + 4;
        let notification_height = 3;
        let notification_x = (area.width.saturating_sub(notification_width)) / 2;
        
        // Animación de deslizamiento desde arriba
        let target_y = 2;
        let start_y = 0;
        let current_y = start_y + ((target_y - start_y) as f32 * progress) as u16;
        
        let notification_area = Rect {
            x: notification_x,
            y: current_y,
            width: notification_width,
            height: notification_height,
        };

        f.render_widget(Clear, notification_area);

        let color = self.notification_type.color();
        let notification_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(color));

        let paragraph = Paragraph::new(self.message.as_str())
            .block(notification_block)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        
        f.render_widget(paragraph, notification_area);
    }
}
