//! The provider reads `Decks.log` next to `Power.log` and gives each game its deck.

use std::fs;

use overlay_hearthstone::Provider;

fn deck_block(time: &str, name: &str, code: &str) -> String {
    [
        "Finding Game With Deck:".to_owned(),
        format!("### {name}"),
        "# Deck ID: 7".to_owned(),
        code.to_owned(),
    ]
    .iter()
    .map(|content| format!("I {time} {content}\n"))
    .collect()
}

fn create_game(time: &str) -> String {
    format!(
        "D {time} GameState.DebugPrintPower() - CREATE_GAME\n\
         D {time} PowerTaskList.DebugPrintPower() -     CREATE_GAME\n"
    )
}

#[test]
fn each_game_gets_the_deck_chosen_before_it() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("Hearthstone_2026_10_17_17_15_14");
    fs::create_dir(&folder).unwrap();
    // Two games read at once, as when overLay starts during a session.
    let decks = [
        deck_block("10:00:00.0000000", "First", "AAECAZ8FAqqKBO2f"),
        deck_block("11:00:00.0000000", "Second", "AAECAf0EApegBIDg"),
    ]
    .concat();
    let power = [
        create_game("10:00:05.0000000"),
        create_game("11:00:05.0000000"),
    ]
    .concat();
    fs::write(folder.join("Decks.log"), decks).unwrap();
    fs::write(folder.join("Power.log"), power).unwrap();

    let mut provider = Provider::new(dir.path());
    provider.poll().unwrap();
    let deck = provider.info().deck.expect("deck of the second game");
    assert_eq!(deck.name, "Second");
    assert_eq!(deck.code, "AAECAf0EApegBIDg");

    // A following game without a chosen deck (loaner deck, Battlegrounds) has none.
    let mut more = fs::read_to_string(folder.join("Power.log")).unwrap();
    more.push_str(&create_game("12:00:05.0000000"));
    fs::write(folder.join("Power.log"), more).unwrap();
    provider.poll().unwrap();
    assert_eq!(provider.info().deck, None);
}
