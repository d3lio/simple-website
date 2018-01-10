use std::collections::HashSet;

pub type Sequence = Vec<char>;

pub enum Error {
    LenMismatch,
    NonUnique
}

pub enum GameState {
    InProgress,
    Win,
    Loss
}

#[derive(Serialize)]
pub struct HistoryEntry {
    sequence: String,
    bulls: u32,
    cows: u32
}

/// Bulls and cows game
pub struct Game {
    sequence: Sequence,
    max_attempts: u32,
    history: Vec<HistoryEntry>,
    state: GameState
}

pub enum GameResult {
    Feedback {
        bulls: u32,
        cows: u32
    },
    Win,
    Loss
}

impl Game {
    pub fn new(sequence: Sequence, max_attempts: u32) -> Result<Self, Error> {
        if unique(sequence.clone()) {
            Ok(Self {
                sequence,
                max_attempts,
                history: Vec::new(),
                state: GameState::InProgress
            })
        } else {
            Err(Error::NonUnique)
        }
    }

    pub fn sequence(&self) -> &Sequence {
        &self.sequence
    }

    pub fn history(&self) -> &Vec<HistoryEntry> {
        &self.history
    }

    pub fn guess(&mut self, sequence: Sequence) -> Result<GameResult, Error> {
        if self.sequence.len() != sequence.len() {
            return Err(Error::LenMismatch)
        }

        if !unique(sequence.clone()) {
            return Err(Error::NonUnique)
        }

        let res = match self.state {
            GameState::InProgress => {
                let feedback = sequence.clone().into_iter().enumerate().fold((0, 0), |acc, (idx, guess)| {
                    if let Some(position) = self.sequence.iter().position(|&symbol| symbol == guess) {
                        if position == idx {
                            (acc.0 + 1, acc.1)
                        } else {
                            (acc.0, acc.1 + 1)
                        }
                    } else {
                        acc
                    }
                });

                self.history.push(HistoryEntry {
                    sequence: sequence.into_iter().collect::<String>(),
                    bulls: feedback.0,
                    cows: feedback.1
                });

                if feedback.0 as usize == self.sequence.len() {
                    self.state = GameState::Win;
                    GameResult::Win
                } else if self.history.len() > self.max_attempts as usize {
                    self.state = GameState::Loss;
                    GameResult::Loss
                } else {
                    GameResult::Feedback{
                        bulls: feedback.0,
                        cows: feedback.1
                    }
                }
            },
            GameState::Win => GameResult::Win,
            GameState::Loss => GameResult::Loss
        };

        Ok(res)
    }
}

fn unique(seq: Sequence) -> bool {
    let len = seq.len();
    seq.into_iter().collect::<HashSet<_>>().len() == len
}
