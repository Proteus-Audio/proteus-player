use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use proteus_lib::playback::player::Player;

#[derive(Clone)]
pub struct Windows {
    // names: HashMap<String, String>,
    players: HashMap<String, Arc<Mutex<Player>>>,
}

impl Windows {
    pub fn new() -> Self {
        Windows {
            // names: HashMap::new(),
            players: HashMap::new(),
        }
    }

    // pub fn list(&self) -> Vec<String> {
    //     self.players.keys().cloned().collect()
    // }

    pub fn add(&mut self, label: String, url: &String) {
        self.players
            .insert(label, Arc::new(Mutex::new(Player::new(url))));
    }

    // pub fn remove(&mut self, label: &str) {
    //     self.players.remove(label);
    // }

    pub fn get(&self, label: &str) -> Option<&Arc<Mutex<Player>>> {
        self.players.get(label)
    }
}
