use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::domain::models::Task;

pub fn draw_task_component(f: &mut Frame, area: Rect, task: &Task, is_selected: bool, is_flipped: bool) {
    // Color del contorno según selección
    let border_color = if is_selected { Color::Blue } else { Color::Gray };
    let text_color = if is_selected { Color::White } else { Color::Gray };
    
    // Contenedor principal con doble borde
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Double);
    
    let inner_area = container_block.inner(area);
    f.render_widget(container_block, area);
    
    if is_flipped {
        // Modo volteado: mostrar descripción o texto por defecto
        let description_text = if task.description.trim().is_empty() {
            "empty description"
        } else {
            &task.description
        };
        
        let description_style = if task.description.trim().is_empty() {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(text_color).add_modifier(Modifier::ITALIC)
        };
        
        let description_paragraph = Paragraph::new(description_text)
            .style(description_style)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(description_paragraph, inner_area);
    } else {
        // Modo normal: layout de tres columnas
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),  // Sección 1: Alert icon
                Constraint::Length(1),  // Separador 1
                Constraint::Min(0),     // Sección 2: Name, Tag, Date
                Constraint::Length(1),  // Separador 2
                Constraint::Length(8),  // Sección 3: Priority
            ])
            .split(inner_area);
        
        // Sección 1: Alert icon
        let alert_icon = if task.alert { "🔔" } else { "🚫" };
        let alert_paragraph = Paragraph::new(alert_icon)
            .style(Style::default().fg(text_color))
            .alignment(Alignment::Center);
        f.render_widget(alert_paragraph, columns[0]);
        
        // Separador 1 (línea vertical completa)
        for row in 0..columns[1].height {
            let separator_area = Rect {
                x: columns[1].x,
                y: columns[1].y + row,
                width: 1,
                height: 1,
            };
            let separator1 = Paragraph::new("│")
                .style(Style::default().fg(border_color));
            f.render_widget(separator1, separator_area);
        }
        
        // Sección 2: Name (primera línea), Tag y Date (segunda línea)
        let finish_date_display = if task.finish_date.trim().is_empty() {
            "empty date"
        } else {
            &task.finish_date
        };
        
        let middle_text = format!("{}\n{} • {}", task.name, task.tag, finish_date_display);
        let middle_paragraph = Paragraph::new(middle_text)
            .style(Style::default().fg(text_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(middle_paragraph, columns[2]);
        
        // Separador 2 (línea vertical completa)
        for row in 0..columns[3].height {
            let separator_area = Rect {
                x: columns[3].x,
                y: columns[3].y + row,
                width: 1,
                height: 1,
            };
            let separator2 = Paragraph::new("│")
                .style(Style::default().fg(border_color));
            f.render_widget(separator2, separator_area);
        }
        
        // Sección 3: Priority con color de fondo
        let priority_color = match task.priority.as_str() {
            "critical" => Color::Red,
            "high" => Color::Rgb(255, 165, 0), // Naranja
            "medium" => Color::Blue,
            "low" => Color::Yellow,
            _ => Color::Gray,
        };
        
        let priority_paragraph = Paragraph::new(task.priority.to_uppercase())
            .style(Style::default().fg(Color::White).bg(priority_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(priority_paragraph, columns[4]);
    }
}
