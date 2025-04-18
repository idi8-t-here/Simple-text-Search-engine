use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{
    error::Error,
    io::{self, Stdout},
    time::{Duration, Instant},
};
use throbber_widgets_tui::Throbber;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
    Frame, Terminal,
};

use runtime::{perform_search, SearchType, Scope, AppMessage};  // Import from our lib

struct App {
    input_scope: String,
    input_type: String,
    input_term: String,
    results: Vec<String>,
    result_state: ListState,
    debug_state: ListState,
    state: AppState,
    status_message: Option<String>,
    debug_messages: Vec<String>,
    throbber_state: throbber_widgets_tui::ThrobberState,
    is_loading: bool,
    loading_start_time: Option<Instant>,
    sender: Sender<AppMessage>,
    receiver: Receiver<AppMessage>,
}

enum AppState {
    ScopeInput,
    TypeInput,
    TermInput,
    ShowResults,
}

impl App {
    fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            input_scope: String::new(),
            input_type: String::new(),
            input_term: String::new(),
            results: Vec::new(),
            result_state: {
                let mut state = ListState::default();
                state.select(Some(0));
                state
            },
            debug_state: {
                let mut state = ListState::default();
                state.select(Some(0));
                state
            },
            state: AppState::ScopeInput,
            status_message: None,
            debug_messages: Vec::new(),
            throbber_state: throbber_widgets_tui::ThrobberState::default(),
            is_loading: false,
            loading_start_time: None,
            sender,
            receiver,
        }
    }

    fn add_debug_message(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.debug_messages
            .push(format!("[{}] {}", timestamp, message));
        if self.debug_messages.len() > 100 {
            self.debug_messages.remove(0);
        }
        self.debug_state
            .select(Some(self.debug_messages.len().saturating_sub(1)));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.add_debug_message("Application started".to_string());

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if app.is_loading {
            app.throbber_state.calc_next();
        }

        // Check for search results from the background thread
        if let Ok(message) = app.receiver.try_recv() {
            match message {
                AppMessage::SearchComplete(results, duration) => {
                    app.results = results.into_iter().map(|(_, s)| s).collect();
                    app.is_loading = false;
                    app.loading_start_time = None;
                    app.result_state.select(Some(0));
                    app.add_debug_message(format!(
                        "Search Completed in => \x1b[1m{:.2?}\x1b[0m",
                        duration
                    ));
                }
                AppMessage::Debug(message) => {
                    app.add_debug_message(message);
                }
            }
        }
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        app.add_debug_message("Exiting application".to_string());
                        break;
                    }
                    KeyCode::Esc => {
                        app.add_debug_message("Status message cleared".to_string());
                        app.status_message = None;
                    }
                    _ => {}
                }

                match app.state {
                    AppState::ScopeInput => handle_scope_input(&mut app, key),
                    AppState::TypeInput => handle_type_input(&mut app, key),
                    AppState::TermInput => handle_term_input(&mut app, key),
                    AppState::ShowResults => match key.code {
                        KeyCode::Down => {
                            if let Some(selected) = app.result_state.selected() {
                                let next = if selected >= app.results.len() - 1 {
                                    selected
                                } else {
                                    selected + 1
                                };
                                app.result_state.select(Some(next));
                                app.add_debug_message(format!("Selected result #{}", next + 1));
                            }
                        }
                        KeyCode::Up => {
                            if let Some(selected) = app.result_state.selected() {
                                let prev = if selected == 0 { 0 } else { selected - 1 };
                                app.result_state.select(Some(prev));
                                app.add_debug_message(format!("Selected result #{}", prev + 1));
                            }
                        }
                        _ => {}
                    },
                }

                match key.code {
                    KeyCode::PageDown => {
                        if let Some(selected) = app.debug_state.selected() {
                            let next =
                                (selected + 10).min(app.debug_messages.len().saturating_sub(1));
                            app.debug_state.select(Some(next));
                        }
                    }
                    KeyCode::PageUp => {
                        if let Some(selected) = app.debug_state.selected() {
                            let prev = selected.saturating_sub(10);
                            app.debug_state.select(Some(prev));
                        }
                    }
                    KeyCode::End => {
                        app.debug_state
                            .select(Some(app.debug_messages.len().saturating_sub(1)));
                    }
                    KeyCode::Home => {
                        app.debug_state.select(Some(0));
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),  // Status message
            Constraint::Length(3),  // Scope input
            Constraint::Length(3),  // Type input
            Constraint::Length(3),  // Term input
            Constraint::Min(1),     // Results or help
            Constraint::Length(10), // Debug window
        ])
        .split(frame.area());

    // Status message
    if let Some(message) = &app.status_message {
        let status = Paragraph::new(message.as_str()).style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[0]);
    }

    // Scope input
    let scope_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Scope (1: Words, 2: Lines)")
        .style(match app.state {
            AppState::ScopeInput => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    frame.render_widget(
        Paragraph::new(app.input_scope.as_str()).block(scope_block),
        chunks[1],
    );

    // Type input
    let type_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Type (1: Prefix, 2: Suffix, 3: Contains)")
        .style(match app.state {
            AppState::TypeInput => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    frame.render_widget(
        Paragraph::new(app.input_type.as_str()).block(type_block),
        chunks[2],
    );

    // Term input
    let term_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Term")
        .style(match app.state {
            AppState::TermInput => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    frame.render_widget(
        Paragraph::new(app.input_term.as_str()).block(term_block),
        chunks[3],
    );

    // Main content area (loading or results/help)
    let main_area = chunks[4];
    match app.state {
        AppState::ShowResults => {
            if app.is_loading {
                let loading_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Searching")
                    .border_style(Style::default().fg(Color::Yellow))
                    .padding(Padding::new(1, 0, 0, 0));
                frame.render_widget(loading_block, main_area);

                let throbber = Throbber::default()
                    .label("Loading results... ")
                    .style(Style::default().fg(Color::Yellow));

                let centered_area = tui::layout::Rect {
                    x: main_area.x + (main_area.width - 20) / 2,
                    y: main_area.y + main_area.height / 2,
                    width: 20,
                    height: 1,
                };

                frame.render_stateful_widget(throbber, centered_area, &mut app.throbber_state);
            } else {
                let items: Vec<ListItem> = app
                    .results
                    .iter()
                    .enumerate()
                    .map(|(i, term)| {
                        let prefix = format!("#{} -> ", i + 1);
                        let term_lower = term.to_lowercase();
                        let search_term = app.input_term.trim().to_lowercase();
                        let is_selected = app.result_state.selected() == Some(i);

                        let line = if is_selected {
                            if let Some(start_idx) = term_lower.find(&search_term) {
                                Line::from(vec![
                                    Span::styled(prefix, Style::default().fg(Color::Green)),
                                    Span::styled(
                                        &term[..start_idx],
                                        Style::default().fg(Color::Green),
                                    ),
                                    Span::styled(
                                        &term[start_idx..start_idx + search_term.len()],
                                        Style::default().fg(Color::LightYellow),
                                    ),
                                    Span::styled(
                                        &term[start_idx + search_term.len()..],
                                        Style::default().fg(Color::Green),
                                    ),
                                ])
                            } else {
                                Line::from(vec![
                                    Span::styled(prefix, Style::default().fg(Color::Green)),
                                    Span::styled(term, Style::default().fg(Color::Green)),
                                ])
                            }
                        } else {
                            Line::from(vec![
                                Span::styled(prefix, Style::default().fg(Color::Green)),
                                Span::styled(term, Style::default().fg(Color::Green)),
                            ])
                        };

                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Results")
                            .border_style(Style::default().fg(Color::Green))
                            .padding(Padding::new(1, 0, 0, 0)),
                    )
                    .highlight_style(Style::default());

                frame.render_stateful_widget(list, chunks[4], &mut app.result_state);
            }
        }
        AppState::ScopeInput | AppState::TypeInput | AppState::TermInput => {
            let help_text = match app.state {
                AppState::ScopeInput => "Enter 1 for Words or 2 for Lines, then press Enter",
                AppState::TypeInput => {
                    "Enter 1 for Prefix, 2 for Suffix, or 3 for Contains, then press Enter"
                }
                AppState::TermInput => "Enter your search term and press Enter",
                _ => "",
            };
            let help = Paragraph::new(help_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Help")
                        .border_style(Style::default().fg(Color::Green))
                        .padding(Padding::new(1, 0, 0, 0)),
                )
                .style(Style::default().fg(Color::Green));
            frame.render_widget(help, main_area);
        }
    }

    // Debug messages
    let debug_messages: Vec<ListItem> = app
        .debug_messages
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();

    let debug_list = List::new(debug_messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Debug Log (PgUp/PgDown/Home/End to scroll)"),
    );

    frame.render_stateful_widget(debug_list, chunks[5], &mut app.debug_state);
}

fn handle_scope_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if app.input_scope.trim() == "1" || app.input_scope.trim() == "2" {
                app.add_debug_message(format!(
                    "Scope set to: {}",
                    if app.input_scope.trim() == "1" {
                        "Words"
                    } else {
                        "Lines"
                    }
                ));
                app.state = AppState::TypeInput;
            }
        }
        KeyCode::Char(c) => {
            app.input_scope.push(c);
        }
        KeyCode::Backspace => {
            app.input_scope.pop();
        }
        _ => {}
    }
}

fn handle_type_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if app.input_type.trim() == "1"
                || app.input_type.trim() == "2"
                || app.input_type.trim() == "3"
            {
                app.add_debug_message(format!(
                    "Search type set to: {}",
                    if app.input_type.trim() == "1" {
                        "Prefix"
                    } else if app.input_type.trim() == "2" {
                        "Suffix"
                    } else {
                        "Contains"
                    }
                ));
                app.state = AppState::TermInput;
            }
        }
        KeyCode::Char(c) => {
            app.input_type.push(c);
        }
        KeyCode::Backspace => {
            app.input_type.pop();
        }
        _ => {}
    }
}

fn handle_term_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if !app.input_term.trim().is_empty() {
                app.add_debug_message(format!(
                    "Searching for term: \x1b[1m{}\x1b[0m",
                    app.input_term.trim()
                ));

                // Set loading state and clear results
                app.is_loading = true;
                app.loading_start_time = Some(Instant::now());
                app.results.clear();
                app.state = AppState::ShowResults;

                let scope = match app.input_scope.trim() {
                    "1" => Scope::Words,
                    "2" => Scope::Lines,
                    _ => return,
                };

                let search_type = match app.input_type.trim() {
                    "1" => SearchType::Prefix,
                    "2" => SearchType::Suffix,
                    "3" => SearchType::Contains,
                    _ => return,
                };

                // Clone all necessary data
                let term = app.input_term.trim().to_string();
                let scope_clone = scope;
                let search_type_clone = search_type;
                let app_sender = app.sender.clone();
                let debug_sender = app.sender.clone();
                let start_time = Instant::now();

                // Perform search in a separate thread
                std::thread::spawn(move || {
                    let results =
                        perform_search(scope_clone, search_type_clone, &term, debug_sender);
                    let duration = start_time.elapsed();
                    app_sender
                        .send(AppMessage::SearchComplete(results, duration))
                        .unwrap();
                });
            }
        }
        KeyCode::Char(c) => {
            app.input_term.push(c);
        }
        KeyCode::Backspace => {
            app.input_term.pop();
        }
        _ => {}
    }
}
