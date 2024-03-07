use rand::Rng;
use ratatui::widgets::ListState;
use std::{error, vec};

use crate::data::{self, Participant};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    // Tabs
    pub tabs: StatefulTabs,

    // Lists
    pub all_participants: StatefulList<Participant>,
    pub all_winners: Vec<Participant>,

    // Spinner
    pub is_spinning: bool,
    pub position: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub spin_winner: Option<Participant>,
}

impl Default for App {
    fn default() -> Self {
        let tab_titles = vec!["Home".to_string(), "Participants".to_string()];

        let participants = data::read_participants_from_file().expect("Failed to read file");

        Self {
            running: true,
            tabs: StatefulTabs::new(tab_titles),
            all_participants: StatefulList::new(participants),
            all_winners: Vec::new(),
            is_spinning: false,
            position: 0.0,
            speed: 0.0,
            acceleration: 0.0,
            spin_winner: None,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set running to false to quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Handles the tick event of the terminal
    pub fn tick(&mut self) {
        self.spin_round()
    }

    pub fn start_spin(&mut self) {
        if self.all_participants.items.is_empty() {
            return;
        }

        let participant_count = self.all_participants.items.len() as f32;

        let min_acceleration = 0.5 / participant_count;
        let max_acceleration = 0.9 / participant_count;

        let mut rng = rand::thread_rng();

        self.acceleration = rng.gen_range(min_acceleration..max_acceleration);
        self.position = 0.0;
        self.speed = 3.0;
        self.spin_winner = None;
        self.is_spinning = true;
    }

    pub fn apply_acceleration(&mut self) {
        self.position += self.speed;
        let i = self.position.floor() as usize;
        self.all_participants.next(i);
        self.position -= i as f32;
        self.speed *= 1.0 - self.acceleration;
    }

    pub fn spin_round(&mut self) {
        if !self.is_spinning {
            return;
        }

        if self.speed > 0.1 {
            self.apply_acceleration();
            return;
        }

        if let Some(winner) = &mut self.all_participants.get_selected() {
            winner.is_winner = true;

            self.spin_winner = Some(winner.clone());
            self.all_winners.push(winner.clone());

            self.stop_spin();
        }
    }

    pub fn stop_spin(&mut self) {
        self.is_spinning = false;
    }

    pub fn reset_spin(&mut self) {
        self.stop_spin();
        self.speed = 0.0;
        self.spin_winner = None;
    }
}

#[derive(Debug)]
pub struct StatefulTabs {
    pub titles: Vec<String>,
    pub active: usize,
}

impl StatefulTabs {
    pub fn new(titles: Vec<String>) -> StatefulTabs {
        StatefulTabs { titles, active: 0 }
    }

    pub fn next_tab(&mut self) {
        self.active = (self.active + 1) % self.titles.len();
    }

    pub fn prev_tab(&mut self) {
        if self.active > 0 {
            self.active -= 1;
        } else {
            self.active = self.titles.len() - 1;
        }
    }
}

#[derive(Debug)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T: std::clone::Clone> StatefulList<T> {
    pub fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self, increment: usize) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + increment) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn get_selected(&mut self) -> Option<T> {
        match self.state.selected() {
            Some(index) => Some(self.items[index].clone()),
            _ => None,
        }
    }

    pub fn remove(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let Some(i) = self.state.selected() else { return };

        self.items.remove(i);
        self.state.select(None);
    }
}
