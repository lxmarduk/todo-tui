use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::Rect;
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::style::palette::material::{GRAY, WHITE};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget};
use ratatui::{Frame, Terminal};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub enum CurrentScreen {
    Main,
    Add,
    Edit,
    Exit,
}

#[derive(Clone)]
pub struct TodoItem {
    pub done: bool,
    pub description: String,
}

impl From<&TodoItem> for ListItem<'_> {
    fn from(value: &TodoItem) -> Self {
        let line = match value.done {
            false => Line::styled(format!(" ☐ {}", value.description), WHITE),
            true => Line::styled(
                format!(" ✓ {}", value.description),
                (GRAY.c500, Modifier::CROSSED_OUT),
            ),
        };
        ListItem::new(line)
    }
}

pub struct AppState {
    pub current_screen: CurrentScreen,
    pub input: Input,
    pub currently_editing: Option<TodoItem>,
    pub edit_index: usize,
    pub todo_list_state: ListState,
    pub items: Vec<TodoItem>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            edit_index: 0,
            todo_list_state: ListState::default(),
            items: vec![],
        }
    }

    pub fn add_item(&mut self, todo_item: TodoItem) {
        self.items.push(todo_item);
    }

    pub fn remove_at(&mut self, index: usize) {
        self.items.remove(index);
    }

    pub fn replace(&mut self, todo_item: TodoItem, index: usize) {
        self.items.remove(index);
        self.items.insert(index, todo_item.clone());
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = AppState::new();
    run_app(&mut terminal, &mut app)?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app_state: &mut AppState) -> io::Result<bool> {
    loop {
        let evt = event::read()?;
        if let Event::Key(key) = evt {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app_state.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('a') => {
                        // Add new item
                        app_state.current_screen = CurrentScreen::Add;
                    }
                    KeyCode::Enter => {
                        // Edit selected
                        if app_state.items.len() > 0 {
                            if let Some(sel_index) = app_state.todo_list_state.selected() {
                                if let Some(e) = Some(app_state.items[sel_index].clone()) {
                                    app_state.input = e.description.clone().into();
                                    app_state.currently_editing = Some(e);
                                }
                                app_state.current_screen = CurrentScreen::Edit;
                            }
                        }
                    }
                    KeyCode::Char('q') => {
                        // Quit
                        app_state.current_screen = CurrentScreen::Exit;
                    }
                    KeyCode::Char(' ') => {
                        // Mark selected
                        if app_state.items.len() > 0 {
                            if let Some(sel_index) = app_state.todo_list_state.selected() {
                                app_state.items[sel_index].done = !app_state.items[sel_index].done;
                            }
                        }
                    }
                    KeyCode::Up => {
                        app_state.todo_list_state.select_previous();
                    }
                    KeyCode::Down => {
                        app_state.todo_list_state.select_next();
                    }
                    _ => {}
                },
                CurrentScreen::Edit => match key.code {
                    KeyCode::Esc => {
                        app_state.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Enter => {
                        if let Some(sel_index) = app_state.todo_list_state.selected() {
                            if let Some(ce) = &app_state.currently_editing {
                                let desc = app_state.input.value_and_reset();
                                app_state.replace(TodoItem { done: ce.done, description: desc }, sel_index);
                            }
                        }
                        app_state.current_screen = CurrentScreen::Main;
                    }
                    _ => {
                        app_state.input.handle_event(&evt);
                    }
                },
                CurrentScreen::Add => match key.code {
                    KeyCode::Esc => {
                        app_state.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Enter => {
                        app_state.items.push(TodoItem {
                            done: false,
                            description: app_state.input.value_and_reset(),
                        });
                        app_state.current_screen = CurrentScreen::Main;
                    }
                    _ => {
                        app_state.input.handle_event(&evt);
                    }
                },
                _ => {}
            };
        }
        match app_state.current_screen {
            CurrentScreen::Add => {
                terminal.draw(|frame| {
                    let _ = add_ui(frame, app_state);
                })?;
            }
            CurrentScreen::Edit => {
                terminal.draw(|frame| {
                    let _ = edit_ui(frame, app_state);
                })?;
            }
            CurrentScreen::Main => {
                terminal.draw(|frame| {
                    let _ = main_ui(frame, app_state);
                })?;
            }
            CurrentScreen::Exit => break,
        };
    }
    Ok(true)
}

fn main_ui(frame: &mut Frame, app_state: &mut AppState) -> io::Result<()> {
    let block = Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::new().fg(SLATE.c500))
        .title(Line::from("TODO").centered().white());
    let items: Vec<ListItem> = app_state
        .items
        .iter()
        .enumerate()
        .map(|(_i, todo_item)| {
            let list_item = ListItem::from(todo_item);
            list_item
        })
        .collect();
    let lis = List::new(items)
        .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD))
        .block(block);
    StatefulWidget::render(
        lis,
        frame.area(),
        frame.buffer_mut(),
        &mut app_state.todo_list_state,
    );
    Ok(())
}

fn edit_ui(frame: &mut Frame, app_state: &mut AppState) -> io::Result<()> {
    if let Some(_edit_item) = &app_state.currently_editing {
        let area = Rect::new(0, 0, frame.area().width.max(3) - 3, 3);
        let scroll = app_state.input.visual_scroll(area.width as usize);
        let input = Paragraph::new(app_state.input.value())
            .style(Style::default())
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Edit item"));
        frame.render_widget(input, area);
        // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
        // end of the input text and one line down from the border to the input line
        let x = app_state.input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1));
    } else {
        app_state.current_screen = CurrentScreen::Main;
    }
    Ok(())
}

fn add_ui(frame: &mut Frame, app_state: &mut AppState) -> io::Result<()> {
    let area = Rect::new(0, 0, frame.area().width.max(3) - 3, 3);
    let scroll = app_state.input.visual_scroll(area.width as usize);
    let input = Paragraph::new(app_state.input.value())
        .style(Style::default())
        .scroll((0, scroll as u16))
        .block(Block::bordered().title("New item"));
    frame.render_widget(input, area);
    // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
    // end of the input text and one line down from the border to the input line
    let x = app_state.input.visual_cursor().max(scroll) - scroll + 1;
    frame.set_cursor_position((area.x + x as u16, area.y + 1));

    Ok(())
}
