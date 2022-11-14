use crate::{
    logic::{voronoi, Game},
    protocol,
};

#[test]
fn voronoi_fill() {
    let request: protocol::Request =
        serde_json::from_str(include_str!("data/wrapped_rivers_and_lakes_opening.json")).unwrap();
    let game = Game::from(&request);

    // Opening should have 20 tiles for each snake.

    let all_scores = voronoi::all(&game);
    for (_snake, score) in all_scores {
        assert_eq!(score, 20);
    }

    assert_eq!(voronoi::me(&game), 20);
}
