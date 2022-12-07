use crate::{
    logic::{scoring::winter, voronoi, Game},
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

#[test]
fn winter_fill() {
    let request: protocol::Request =
        serde_json::from_str(include_str!("data/wrapped_rivers_and_lakes_opening.json")).unwrap();
    let game = Game::from(&request);

    let all_scores = winter::winter_floodfill::<{ u16::MAX }>(&game);
    for (i, score) in all_scores[0..4].iter().enumerate() {
        println!("{:?}", score);
        if i == 1 {
            // Because of where the food is positioned, snake 1 can claim one more tile at the start
            assert_eq!(score.tile_count, 21);
        } else {
            assert_eq!(score.tile_count, 20);
        }
        assert_eq!(score.food_count, 1);
    }
}
