use std::{
    error::Error, 
    fs, 
    io::{self, Stdout}, 
    path::Path, 
    time::Duration
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    text::Span,
    Terminal,
    style::{Style, Color},
    Frame,
};
use bincode::config;
use levenshtein::levenshtein;
use data_structs::trees::trie::Trie;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
enum SearchType {
    Prefix,
    Suffix,
    Contains
}

#[derive(Debug)]
enum Scope {
    Words,
    Lines,
}

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
}

enum AppState {
    ScopeInput,
    TypeInput,
    TermInput,
    ShowResults,
}

impl App {
    fn add_debug_message(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.debug_messages.push(format!("[{}] {}", timestamp, message));
        if self.debug_messages.len() > 100 {
            self.debug_messages.remove(0);
        }
        // Auto-scroll to bottom when new messages arrive
        self.debug_state.select(Some(self.debug_messages.len().saturating_sub(1)));
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

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    let mut app = App {
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
    };

    app.add_debug_message("Application started".to_string());

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        app.add_debug_message("Exiting application".to_string());
                        break;
                    },
                    KeyCode::Esc => {
                        app.add_debug_message("Status message cleared".to_string());
                        app.status_message = None;
                    }
                    _ => {}
                }
                
                // Handle main application state
                match app.state {
                    AppState::ScopeInput => handle_scope_input(&mut app, key),
                    AppState::TypeInput => handle_type_input(&mut app, key),
                    AppState::TermInput => handle_term_input(&mut app, key),
                    AppState::ShowResults => {
                        match key.code {
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
                                    let prev = if selected == 0 { 
                                        0 
                                    } else { 
                                        selected - 1 
                                    };
                                    app.result_state.select(Some(prev));
                                    app.add_debug_message(format!("Selected result #{}", prev + 1));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                // Handle debug window scrolling (available in all states)
                match key.code {
                    KeyCode::PageDown => {
                        if let Some(selected) = app.debug_state.selected() {
                            let next = (selected + 10).min(app.debug_messages.len().saturating_sub(1));
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
                        app.debug_state.select(Some(app.debug_messages.len().saturating_sub(1)));
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

fn ui<B: tui::backend::Backend>(frame: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),   // Status message
            Constraint::Length(3),   // Scope input
            Constraint::Length(3),   // Type input
            Constraint::Length(3),   // Term input
            Constraint::Min(1),      // Results or help
            Constraint::Length(10),  // Debug window
        ])
        .split(frame.size());

    // Status message
    if let Some(message) = &app.status_message {
        let status = Paragraph::new(message.as_str())
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[0]);
    }

    // Input fields - each with conditional border color
    let scope_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Scope (1: Words, 2: Lines)")
        .style(match app.state {
            AppState::ScopeInput => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    let scope_input = Paragraph::new(app.input_scope.as_str())
        .block(scope_block);
    frame.render_widget(scope_input, chunks[1]);

    let type_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Type (1: Prefix, 2: Suffix, 3: Contains)")
        .style(match app.state {
            AppState::TypeInput => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    let type_input = Paragraph::new(app.input_type.as_str())
        .block(type_block);
    frame.render_widget(type_input, chunks[2]);

    let term_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Term")
        .style(match app.state {
            AppState::TermInput => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    let term_input = Paragraph::new(app.input_term.as_str())
        .block(term_block);
    frame.render_widget(term_input, chunks[3]);

    // Main content area
    match app.state {
        AppState::ShowResults => {
            let items: Vec<ListItem> = app.results.iter()
                .enumerate()
                .map(|(i, term)| {
                    ListItem::new(format!("#{} -> {}", i + 1, term))
                        .style(Style::default().fg(Color::Yellow))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Results (Up/Down to scroll, CTRL+C to quit)")
                    .style(Style::default().fg(Color::Green)))
                .highlight_style(Style::default().fg(Color::LightGreen));
            frame.render_stateful_widget(list, chunks[4], &mut app.result_state);
        }
        _ => {
            let help_block = Block::default()
                .borders(Borders::ALL)
                .title("Help");
            let help_text = match app.state {
                AppState::ScopeInput => "Enter 1 for Words or 2 for Lines, then press Enter",
                AppState::TypeInput => "Enter 1 for Prefix, 2 for Suffix, or 3 for Contains, then press Enter",
                AppState::TermInput => "Enter your search term and press Enter",
                _ => "",
            };
            let styled_text = Span::styled(
                help_text,
                Style::default().fg(Color::Green)
            );
            let help = Paragraph::new(styled_text).block(help_block);
            frame.render_widget(help, chunks[4]);
        }
    }

    // Debug window
    let debug_messages: Vec<ListItem> = app.debug_messages.iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();
    
    let debug_list = List::new(debug_messages)
        .block(Block::default().borders(Borders::ALL).title("Debug Log (PgUp/PgDown/Home/End to scroll)"));
    
    frame.render_stateful_widget(debug_list, chunks[5], &mut app.debug_state);
}

fn handle_scope_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if app.input_scope.trim() == "1" || app.input_scope.trim() == "2" {
                app.add_debug_message(format!("Scope set to: {}", if app.input_scope.trim() == "1" {"Words"} else {"Lines"}));
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
            if app.input_type.trim() == "1" || app.input_type.trim() == "2" || app.input_type.trim() == "3" {
                app.add_debug_message(format!("Search type set to: {}", if app.input_type.trim() == "1" {"Prefix"} else if app.input_type.trim() == "2" {"Suffix"} else {"Contains"}));
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
                app.add_debug_message(format!("Searching for term: {}", app.input_term.trim()));
                
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

                app.results = perform_search(app, scope, search_type, app.input_term.trim().to_string())
                    .into_iter()
                    .map(|(_, s)| s)
                    .collect();

                app.add_debug_message(format!("Found {} results", app.results.len()));
                app.result_state.select(Some(0));
                app.state = AppState::ShowResults;
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

fn perform_search(app: &mut App, scope: Scope, search_type: SearchType, term: String) -> Vec<(u8,String)> {
    let mut sorted_result: Vec<(u8,String)> = Vec::new();
    let scope_path = match scope {
        Scope::Words => "word_scope",
        Scope::Lines => "line_scope",
    };

    let type_path = match search_type {
        SearchType::Prefix => "trie-serial.bin",
        SearchType::Suffix => "suffix-serial.bin",
        SearchType::Contains => "inverted-serial.bin",
    };

    let path = format!("./serialized_outputs/{}/{}", scope_path, type_path);
    let file_path = Path::new(&path);
    
    app.add_debug_message(format!("Searching in file: {}", path));
    
    if let Ok(contents) = fs::read(file_path) {
        app.add_debug_message("File read successfully".to_string());
        if let Ok((trie, _)) = bincode::decode_from_slice::<Trie, _>(&contents, config::standard()) {
            app.add_debug_message("Trie decoded successfully".to_string());
            if let Ok(search_results) = trie.search(term.to_string()) {
                let results = search_results.unwrap();
                
                for item in results.iter() {
                    if matches!(scope, Scope::Lines) {
                        let lines_scope = item.unicode_words().collect::<Vec<&str>>();
                        if let Some(first_word) = lines_scope.first() {
                            if *first_word.to_lowercase() == term.to_lowercase() {
                                app.add_debug_message(format!("MATCH: first_word='{}', term='{}', full_line='{}'", 
                                    first_word, term, item));
                                    let priority = levenshtein(&term, item);
                                    sorted_result.push((priority as u8, item.to_string()));
                            } else {
                                app.add_debug_message(format!("FAILED: first_word='{}', term='{}', full_line='{}'", 
                                    first_word, term, item));
                            }
                        } else {
                            app.add_debug_message(format!("Empty line or no words found in: '{}'", item));
                        }
                    } else {
                        let priority = levenshtein(&term, item);
                        sorted_result.push((priority as u8, item.to_string()));
                    }
                }
            } else {
                app.add_debug_message("Search failed".to_string());
            }
        } else {
            app.add_debug_message("Failed to decode trie".to_string());
        }
    } else {
        app.add_debug_message("Failed to read file".to_string());
    }

    sorted_result.sort_by_key(|(priority, _)| *priority);
    sorted_result.truncate(100);
    app.add_debug_message(format!("Final results count: {}", sorted_result.len()));
    sorted_result
}
