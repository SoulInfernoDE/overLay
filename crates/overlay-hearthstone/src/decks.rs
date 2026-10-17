//! Reads which deck the user plays from `Decks.log`.
//!
//! When a game is searched with one of the user's own decks, Hearthstone writes
//!
//! ```text
//! I 17:20:45.9193400 Finding Game With Deck:
//! I 17:20:45.9193400 ### Paladin
//! I 17:20:45.9193400 # Deck ID: 2772580478
//! I 17:20:45.9193400 AAECAZ8FAqqKBO2fBA6cnwShnwSo…
//! ```
//!
//! The last line is the deck code (the format Hearthstone imports and exports). Loaner
//! decks and modes without a deck (Battlegrounds) write nothing. The same block after
//! `Deck Contents Received:` lists every deck of the account at login and is not a
//! choice.

use serde::Serialize;

/// A deck chosen for the next game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Deck {
    pub name: String,
    pub deck_id: Option<u64>,
    /// Deck code, as Hearthstone exports it.
    pub code: String,
}

#[derive(Debug, Default)]
pub struct DeckLog {
    /// The block being read after `Finding Game With Deck:`.
    reading: Option<Deck>,
}

impl DeckLog {
    /// Takes one line of `Decks.log`; returns the deck once its block is complete.
    pub fn apply(&mut self, line: &str) -> Option<Deck> {
        let content = content(line)?;
        if content == "Finding Game With Deck:" {
            self.reading = Some(Deck::default());
            return None;
        }
        let deck = self.reading.as_mut()?;
        if let Some(name) = content.strip_prefix("### ") {
            deck.name = name.to_owned();
        } else if let Some(id) = content.strip_prefix("# Deck ID: ") {
            deck.deck_id = id.trim().parse().ok();
        } else if is_deck_code(content) {
            deck.code = content.to_owned();
            return self.reading.take();
        } else {
            // Anything else ends the block without a code.
            self.reading = None;
        }
        None
    }
}

fn content(line: &str) -> Option<&str> {
    let rest = line.get(2..)?;
    rest.split_once(' ').map(|(_, content)| content.trim_end())
}

fn is_deck_code(text: &str) -> bool {
    text.len() >= 8
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(content: &str) -> String {
        format!("I 17:20:45.9193400 {content}")
    }

    #[test]
    fn reads_the_chosen_deck() {
        let mut log = DeckLog::default();
        let lines = [
            "Finding Game With Deck:",
            "### Paladin",
            "# Deck ID: 2772580478",
            "AAECAZ8FAqqKBO2fBA6cnwShnwSonwTr==",
        ];
        let mut found = None;
        for content in lines {
            if let Some(deck) = log.apply(&line(content)) {
                found = Some(deck);
            }
        }
        assert_eq!(
            found,
            Some(Deck {
                name: "Paladin".into(),
                deck_id: Some(2772580478),
                code: "AAECAZ8FAqqKBO2fBA6cnwShnwSonwTr==".into(),
            })
        );
    }

    #[test]
    fn ignores_the_deck_list_at_login() {
        let mut log = DeckLog::default();
        for content in [
            "Deck Contents Received:",
            "### Magier",
            "# Deck ID: 1",
            "AAECAf0EApegBIDgBw78ngT9ngSm==",
        ] {
            assert_eq!(log.apply(&line(content)), None);
        }
    }
}
