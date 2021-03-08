
#[allow(dead_code)]
mod util;

use util::{
    event::{Event, Events},
    StatefulList,
};
use std::{error::Error, io};
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{env, fs};
use serde_derive::Deserialize;
use toml;

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
struct App {
    items: StatefulList<(String, Job)>,
}

impl App {
    fn new(config: Config) -> App {
        let items = config.job.into_iter().map(|job| (job.name.clone(), job)).collect();
        let mut items_list = StatefulList::with_items(items);
        items_list.next();
        App {
            items: items_list,
        }
    }

    /// This only exists to simulate some kind of "progress"
    fn update(&mut self) {
        
    }
}

#[derive(Debug, Deserialize)]
struct BazelJvmBinary {
    target: String,
}

#[derive(Debug, Deserialize)]
struct YarnScript {
    workspace: String,
    script: String
}

#[derive(Debug, Deserialize)]
struct Job {
    name: String,
    bazel: Option<BazelJvmBinary>,
    yarn: Option<YarnScript>,
    dependencies: Option<Vec<String>>
}

#[derive(Debug, Deserialize)]
struct Config {
    job: Vec<Job>,
}

fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();
    let config_filename = &args[1];
    let config_contents = fs::read_to_string(config_filename)?;
    let config: Config = toml::from_str(&config_contents)?;

    println!("Got Config: {:?}", config);
    // panic!("stop here");

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    // Create a new app with some exapmle state
    let mut app = App::new(config);

    loop {
        terminal.draw(|f| {
            // Create two chunks with equal horizontal screen space
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            // Iterate through all elements in the `items` app and append some debug text to it.
            let items: Vec<ListItem> = app
                .items
                .items
                .iter()
                .map(|(name, job)| {
                    let mut lines = vec![Spans::from(Span::raw(name))];
                    // lines.push();
                    // for _ in 0..i.1 {
                        lines.push(Spans::from(Span::styled(
                            format!("{:?}", job),
                            Style::default().add_modifier(Modifier::ITALIC),
                        )));
                    // }
                    ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
                })
                .collect();

            // Create a List from all list items and highlight the currently selected one
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("List"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            // We can now render the item list
            f.render_stateful_widget(items, chunks[0], &mut app.items.state);

            let status_message = if let Some(idx) = app.items.state.selected() {
                let (_, active_job) = &app.items.items[idx];
                format!("Hello, {:?}", active_job)
            } else {
                "No selected job".to_string()
            };

            let right_panel = Paragraph::new(status_message).block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(right_panel, chunks[1]);
        })?;

        // This is a simple example on how to handle events
        // 1. This breaks the loop and exits the program on `q` button press.
        // 2. The `up`/`down` keys change the currently selected item in the App's `items` list.
        // 3. `left` unselects the current item.
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                // Key::Left => {
                //     app.items.unselect();
                // }
                Key::Down => {
                    app.items.next();
                }
                Key::Up => {
                    app.items.previous();
                }
                _ => {}
            },
            Event::Tick => {
                app.update();
            }
        }
    }

    Ok(())
}