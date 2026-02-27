use crate::core::ToolInfo;
use crate::interfaces::tui::widget::{SizeConstraints, WidgetCapabilities};
use crate::interfaces::tui::{
    theme::Theme, widget::WidgetError, Action, Widget, WidgetContext, WidgetId,
};
use crate::tools::ToolRegistry;
use async_trait::async_trait;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use serde_json::Value;
use std::sync::Arc;

pub struct ToolManagerWidget {
    registry: Arc<ToolRegistry>,
    state: ListState,
    tools: Vec<ToolInfo>,
    filter: String,
    show_details: bool,

    // Widget trait fields
    id: WidgetId,
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
}

struct FilterSet {
    category_filter: Option<String>,
    tag_filter: Option<String>,
    text_filter: Option<String>,
}

impl ToolManagerWidget {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        let tools = registry.list_tools();
        let mut state = ListState::default();
        if !tools.is_empty() {
            state.select(Some(0));
        }

        Self {
            registry,
            state,
            tools,
            filter: String::new(),
            show_details: false,

            id: WidgetId::from("tool_manager"),
            context: WidgetContext::new(WidgetId::from("tool_manager")),
            capabilities: WidgetCapabilities::default(),
            size_constraints: SizeConstraints::default(),
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.refresh_tools();
                true
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.refresh_tools();
                true
            }
            KeyCode::Down => {
                self.next();
                true
            }
            KeyCode::Up => {
                self.previous();
                true
            }
            KeyCode::Enter => {
                self.show_details = !self.show_details;
                true
            }
            KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                    true
                } else if !self.filter.is_empty() {
                    self.filter.clear();
                    self.refresh_tools();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tools.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tools.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn refresh_tools(&mut self) {
        let all_tools = self.registry.list_tools();
        let filter_set = self.parse_filter(&self.filter);

        self.tools = all_tools
            .into_iter()
            .filter(|t| self.matches_filter(t, &filter_set))
            .collect();

        if self.tools.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
    }

    fn parse_filter(&self, filter: &str) -> FilterSet {
        let mut category_filter = None;
        let mut tag_filter = None;
        let mut text_parts = Vec::new();

        for part in filter.split_whitespace() {
            if part.starts_with("cat:") {
                category_filter = Some(part[4..].to_string());
            } else if part.starts_with("tag:") {
                tag_filter = Some(part[4..].to_string());
            } else {
                text_parts.push(part);
            }
        }

        FilterSet {
            category_filter,
            tag_filter,
            text_filter: if text_parts.is_empty() {
                None
            } else {
                Some(text_parts.join(" "))
            },
        }
    }

    fn matches_filter(&self, tool: &ToolInfo, filter_set: &FilterSet) -> bool {
        if let Some(cat_filter) = &filter_set.category_filter {
            if !tool
                .category
                .as_ref()
                .is_some_and(|cat| cat.to_lowercase().contains(&cat_filter.to_lowercase()))
            {
                return false;
            }
        }

        if let Some(tag_filter) = &filter_set.tag_filter {
            if !tool
                .tags
                .iter()
                .any(|tag: &String| tag.to_lowercase().contains(&tag_filter.to_lowercase()))
            {
                return false;
            }
        }

        if let Some(text_filter) = &filter_set.text_filter {
            let filter_lower = text_filter.to_lowercase();
            if tool.name.to_lowercase().contains(&filter_lower)
                || tool.description.to_lowercase().contains(&filter_lower)
            {
                return true;
            }
            if tool
                .tags
                .iter()
                .any(|tag: &String| tag.to_lowercase().contains(&filter_lower))
            {
                return true;
            }
            return false;
        }

        true
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);

        self.render_list(frame, chunks[0]);
        self.render_details(frame, chunks[1]);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .tools
            .iter()
            .map(|tool| {
                let mut spans = vec![Span::styled(
                    &tool.name,
                    Style::default().add_modifier(Modifier::BOLD),
                )];

                if let Some(cat) = &tool.category {
                    spans.push(Span::raw(format!(" ({})", cat)));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let title = if self.filter.is_empty() {
            "Tools".to_string()
        } else {
            format!("Tools (filter: {})", self.filter)
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_details(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Details");

        if let Some(index) = self.state.selected() {
            if let Some(tool) = self.tools.get(index) {
                let mut details = vec![
                    Line::from(vec![
                        Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&tool.name),
                    ]),
                    Line::from(vec![
                        Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&tool.version),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "Description: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&tool.description),
                    ]),
                ];

                if let Some(cat) = &tool.category {
                    details.push(Line::from(vec![
                        Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(cat),
                    ]));
                }

                if !tool.tags.is_empty() {
                    details.push(Line::from(vec![
                        Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(tool.tags.join(", ")),
                    ]));
                }

                details.push(Line::from(Span::raw("")));
                details.push(Line::from(Span::styled(
                    "Parameters:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));

                // Simple parameter rendering
                if let Some(properties) = tool.parameters_schema.get("properties").and_then(
                    |v: &Value| -> Option<&serde_json::Map<String, Value>> { v.as_object() },
                ) {
                    let example_params: Vec<String> = properties
                        .iter()
                        .take(2) // Show first 2 parameters as example
                        .map(|(name, schema): (&String, &Value)| {
                            let example_value =
                                match schema.get("type").and_then(|v: &Value| v.as_str()) {
                                    Some("string") => "\"example\"".to_string(),
                                    Some("number") | Some("integer") => "42".to_string(),
                                    Some("boolean") => "true".to_string(),
                                    Some("array") => "[]".to_string(),
                                    Some("object") => "{}".to_string(),
                                    _ => "null".to_string(),
                                };
                            format!("{}: {}", name, example_value)
                        })
                        .collect();

                    if !example_params.is_empty() {
                        details.push(Line::from(Span::raw(format!(
                            "  {{ {}, ... }}",
                            example_params.join(", ")
                        ))));
                    } else {
                        details.push(Line::from(Span::raw("  No parameters")));
                    }
                } else {
                    details.push(Line::from(Span::raw("  No parameters schema")));
                }

                let paragraph = Paragraph::new(details)
                    .block(block)
                    .wrap(Wrap { trim: true });
                frame.render_widget(paragraph, area);
                return;
            }
        }

        let paragraph = Paragraph::new("Select a tool to view details")
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}

#[async_trait]
impl Widget for ToolManagerWidget {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn title(&self) -> &str {
        "Tool Manager"
    }

    fn context(&self) -> &WidgetContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut WidgetContext {
        &mut self.context
    }

    fn capabilities(&self) -> &WidgetCapabilities {
        &self.capabilities
    }

    fn size_constraints(&self) -> &SizeConstraints {
        &self.size_constraints
    }

    async fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        _theme: &Theme,
    ) -> Result<(), WidgetError> {
        self.render(frame, area);
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<Option<Action>, WidgetError> {
        if let Event::Key(key) = event {
            self.handle_input(key);
        }
        Ok(None)
    }
}
