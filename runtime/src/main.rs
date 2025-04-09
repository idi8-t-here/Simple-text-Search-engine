use std::{
    error::Error, fs, io::{self, Stdout}, path::Path, time::Duration
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
    Terminal,
    style::{Style, Color},
    Frame,
};
use bincode::config;
use levenshtein::levenshtein;
use data_structs::trees::trie::Trie;

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
    state: AppState,
}

enum AppState {
    ScopeInput,
    TypeInput,
    TermInput,
    ShowResults,
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
        state: AppState::ScopeInput,
    };

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::ScopeInput => handle_scope_input(&mut app, key),
                    AppState::TypeInput => handle_type_input(&mut app, key),
                    AppState::TermInput => handle_term_input(&mut app, key),
                    AppState::ShowResults => {
                        match key.code {
                            KeyCode::Char('q') |
                            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => break,
                            KeyCode::Down => {
                                if let Some(selected) = app.result_state.selected() {
                                    let next = if selected >= app.results.len() - 1 { selected } else { selected + 1 };
                                    app.result_state.select(Some(next));
                                }
                            }
                            KeyCode::Up => {
                                if let Some(selected) = app.result_state.selected() {
                                    let prev = if selected == 0 { 0 } else { selected - 1 };
                                    app.result_state.select(Some(prev));
                                }
                            }
                            _ => {}
                        }
                    }
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
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(frame.size());

    let scope_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Scope (1: Words, 2: Lines)");
    let scope_input = Paragraph::new(app.input_scope.as_str())
        .block(scope_block);
    frame.render_widget(scope_input, chunks[0]);

    let type_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Type (1: Prefix, 2: Suffix, 3: Contains)");
    let type_input = Paragraph::new(app.input_type.as_str())
        .block(type_block);
    frame.render_widget(type_input, chunks[1]);

    let term_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Term");
    let term_input = Paragraph::new(app.input_term.as_str())
        .block(term_block);
    frame.render_widget(term_input, chunks[2]);

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
                .block(Block::default().borders(Borders::ALL).title("Results (Up/Down to scroll, q to quit)"))
                .highlight_style(Style::default().fg(Color::LightGreen));
            frame.render_stateful_widget(list, chunks[3], &mut app.result_state);
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
            let help = Paragraph::new(help_text).block(help_block);
            frame.render_widget(help, chunks[3]);
        }
    }
}

fn handle_scope_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if app.input_scope.trim() == "1" || app.input_scope.trim() == "2" {
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

                app.results = perform_search(scope, search_type, app.input_term.trim().to_string())
                    .into_iter()
                    .map(|(_, s)| s)
                    .collect();

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

fn perform_search(scope: Scope, search_type: SearchType, term: String) -> Vec<(u8,String)> {
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
    //let mut results = Vec::new();
    
    if let Ok(contents) = fs::read(file_path) {
        if let Ok((trie, _)) = bincode::decode_from_slice::<Trie, _>(&contents, config::standard()) {
            if let Ok(search_results) = trie.search(term.to_string()) {
                let results = search_results.unwrap();
                for item in results.iter() {
                    let priority = levenshtein(&term, item);
                    sorted_result.push((priority as u8, item.to_string()));
                }
            }
        }
    }

    sorted_result.sort_by_key(|(priority, _)| *priority);
    sorted_result.truncate(100);
    sorted_result
}
