use crate::domain::models::ticket::Ticket;
use crate::domain::models::comment::Comment;
use crate::ui::theme::Theme;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Ticket detail widget
pub struct TicketDetail<'a> {
    ticket: &'a Ticket,
    comments: &'a [Comment],
    theme: &'a Theme,
}

impl<'a> TicketDetail<'a> {
    pub fn new(ticket: &'a Ticket, comments: &'a [Comment], theme: &'a Theme) -> Self {
        Self { ticket, comments, theme }
    }

    /// Render the ticket detail view
    pub fn render(self, frame: &mut Frame, area: Rect) {
        // Split horizontally: left for ticket details, right for comments
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Left: ticket details
                Constraint::Percentage(40), // Right: comments sidebar
            ])
            .split(area);

        // Render ticket details on the left
        self.render_ticket_details(frame, horizontal_chunks[0]);
        
        // Render comments sidebar on the right
        self.render_comments(frame, horizontal_chunks[1]);
    }

    /// Render ticket details (left side)
    fn render_ticket_details(&self, frame: &mut Frame, area: Rect) {
        // Split into sections: header, fields, description, metadata
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header (key, summary)
                Constraint::Length(6), // Fields (status, assignee, priority, type)
                Constraint::Min(5),    // Description (flexible)
                Constraint::Length(2), // Metadata (created, updated)
            ])
            .split(area);

        self.render_header(frame, chunks[0]);
        self.render_fields(frame, chunks[1]);
        self.render_description(frame, chunks[2]);
        self.render_metadata(frame, chunks[3]);
    }

    /// Render comments sidebar (right side)
    fn render_comments(&self, frame: &mut Frame, area: Rect) {
        if self.comments.is_empty() {
            let paragraph = Paragraph::new("No comments")
                .style(self.theme.normal)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Comments ({})", self.comments.len())),
                );
            frame.render_widget(paragraph, area);
            return;
        }

        // Create list items for comments
        let items: Vec<ListItem> = self
            .comments
            .iter()
            .map(|comment| {
                let author_name = &comment.author.display_name;
                let created_str = format_date(&comment.created);
                let body_preview = if comment.body.len() > 50 {
                    format!("{}...", &comment.body[..50])
                } else {
                    comment.body.clone()
                };

                // Create a multi-line item
                let lines = vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{} - ", author_name),
                            self.theme.focused,
                        ),
                        Span::styled(created_str, self.theme.normal),
                    ]),
                    Line::from(vec![Span::styled(body_preview, self.theme.normal)]),
                ];

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Comments ({})", self.comments.len())),
            )
            .style(self.theme.normal);

        frame.render_widget(list, area);
    }

    /// Render header with key and summary
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let key_style = self.theme.focused;
        let summary_style = self.theme.normal;

        let header_text = format!("{} - {}", self.ticket.key, self.ticket.summary);
        let paragraph = Paragraph::new(header_text)
            .style(summary_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Ticket Details")
                    .title_style(key_style),
            );

        frame.render_widget(paragraph, area);
    }

    /// Render ticket fields (status, assignee, priority, type)
    fn render_fields(&self, frame: &mut Frame, area: Rect) {
        let status_category_str = match self.ticket.status.category {
            crate::domain::models::ticket::StatusCategory::ToDo => "new",
            crate::domain::models::ticket::StatusCategory::InProgress => "indeterminate",
            crate::domain::models::ticket::StatusCategory::Done => "done",
        };
        let status_style = self.theme.status_style(status_category_str);

        let priority_str = format!("{:?}", self.ticket.priority);
        let priority_style = self.theme.priority_style(&priority_str);

        let assignee_name = self
            .ticket
            .assignee
            .as_ref()
            .map(|u| u.display_name.clone())
            .unwrap_or_else(|| "Unassigned".to_string());

        let fields_text = vec![
            Line::from(vec![
                Span::styled("Status: ", self.theme.normal),
                Span::styled(
                    format!("{}", self.ticket.status.name),
                    status_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("Priority: ", self.theme.normal),
                Span::styled(priority_str, priority_style),
            ]),
            Line::from(vec![
                Span::styled("Type: ", self.theme.normal),
                Span::styled(self.ticket.issue_type.clone(), self.theme.normal),
            ]),
            Line::from(vec![
                Span::styled("Assignee: ", self.theme.normal),
                Span::styled(assignee_name, self.theme.normal),
            ]),
            Line::from(vec![
                Span::styled("Project: ", self.theme.normal),
                Span::styled(self.ticket.project_key.clone(), self.theme.normal),
            ]),
        ];

        let paragraph = Paragraph::new(fields_text)
            .block(Block::default().borders(Borders::ALL).title("Fields"));

        frame.render_widget(paragraph, area);
    }

    /// Render description
    fn render_description(&self, frame: &mut Frame, area: Rect) {
        let description_text = self
            .ticket
            .description
            .as_ref()
            .map(|d| d.as_str())
            .unwrap_or("No description provided.");

        let paragraph = Paragraph::new(description_text)
            .style(self.theme.normal)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Description"),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Render metadata (created, updated)
    fn render_metadata(&self, frame: &mut Frame, area: Rect) {
        let created_str = format_date(&self.ticket.created);
        let updated_str = format_date(&self.ticket.updated);

        let metadata_text = vec![
            Line::from(vec![
                Span::styled("Created: ", self.theme.normal),
                Span::styled(created_str, self.theme.normal),
            ]),
            Line::from(vec![
                Span::styled("Updated: ", self.theme.normal),
                Span::styled(updated_str, self.theme.normal),
            ]),
        ];

        let paragraph = Paragraph::new(metadata_text)
            .block(Block::default().borders(Borders::ALL).title("Metadata"));

        frame.render_widget(paragraph, area);
    }
}

/// Format a datetime for display
fn format_date(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ticket::{Priority, Status, StatusCategory};
    use chrono::Utc;

    fn create_test_ticket() -> Ticket {
        Ticket {
            id: "12345".to_string(),
            key: "TEST-123".to_string(),
            summary: "Test ticket summary".to_string(),
            status: Status {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: StatusCategory::ToDo,
            },
            assignee: None,
            priority: Priority::Medium,
            issue_type: "Task".to_string(),
            project_key: "TEST".to_string(),
            description: Some("This is a test description.".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
        }
    }

    #[test]
    fn test_ticket_detail_creation() {
        let ticket = create_test_ticket();
        let comments = Vec::new();
        let theme = Theme::default();
        let detail = TicketDetail::new(&ticket, &comments, &theme);
        
        assert_eq!(detail.ticket.key, "TEST-123");
    }

    #[test]
    fn test_format_date() {
        let dt = Utc::now();
        let formatted = format_date(&dt);
        assert!(formatted.contains("UTC"));
    }
}
