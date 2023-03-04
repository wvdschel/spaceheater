use crate::{logic::Game, protocol};

#[test]
fn snail_mode_apply_moves() {
    let request: protocol::Request =
        serde_json::from_str(include_str!("data/snail_mode_before.json")).unwrap();

    let before_game = Game::from(&request);
    let mut predicted_game = before_game.clone();

    predicted_game.execute_moves(protocol::Direction::Left, &vec![protocol::Direction::Down]);

    let request: protocol::Request =
        serde_json::from_str(include_str!("data/snail_mode_after.json")).unwrap();

    let after_game = Game::from(&request);

    println!("before: {}", before_game);
    println!("predicted: {}", predicted_game);
    println!("after: {}", after_game);

    assert_eq!(predicted_game.board, after_game.board)
}
