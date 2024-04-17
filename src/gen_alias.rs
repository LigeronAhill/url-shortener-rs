use crate::config::Configuration;
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone)]
pub struct Generator {
    alias_length: usize,
}
impl Generator {
    pub fn new(config: &Configuration) -> Self {
        Self {
            alias_length: config.alias_length as usize,
        }
    }
    pub fn generate_alias(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.alias_length)
            .map(char::from)
            .collect()
    }
}
