use std::cmp;

use crate::logic::{board::MAX_HAZARDS, Board, Point, Tile};

#[test]
fn empty_board() {
    let b = Board::new(11, 11);
    assert_eq!(b.width(), 11);
    assert_eq!(b.height(), 11);
    for x in 0..11 {
        for y in 0..11 {
            assert_eq!(b.get(&Point { x, y }), Tile::Empty)
        }
    }
}

#[test]
fn board_set() {
    for t in [
        Tile::Food,
        Tile::Snake,
        Tile::Head,
        Tile::Empty,
        Tile::Hazard(1),
        Tile::Hazard(2),
        Tile::Hazard(3),
        Tile::HazardWithFood(1),
        Tile::HazardWithFood(2),
        Tile::HazardWithFood(3),
        Tile::HazardWithSnake(1),
        Tile::HazardWithSnake(2),
        Tile::HazardWithSnake(3),
        Tile::HazardWithHead(1),
        Tile::HazardWithHead(2),
        Tile::HazardWithHead(3),
    ] {
        let mut b = Board::new(11, 11);
        assert_eq!(b.width(), 11);
        assert_eq!(b.height(), 11);
        for x in 0..11 {
            for y in 0..11 {
                let p = &Point { x, y };
                assert_eq!(b.get(p), Tile::Empty);
                b.set(p, t);
                let check = b.get(p);
                assert_eq!(
                    check, t,
                    "tried to set {} to {:?}, but read back {:?} - board = {:?}",
                    p, t, check, b
                );

                for x in 0..11 {
                    for y in 0..11 {
                        let p2 = &Point { x, y };
                        if p != p2 {
                            let t2 = b.get(p2);
                            assert_eq!(
                                t2,
                                Tile::Empty,
                                "setting {} to {:?}, checking {} for damage: value is {:?} - board = {:?}",
                                p,
                                t,
                                p2,
                                t2,
                                b,
                            );
                        }
                    }
                }

                b.set(p, Tile::Empty);

                for x in 0..11 {
                    for y in 0..11 {
                        let p2 = &Point { x, y };
                        assert_eq!(b.get(p2), Tile::Empty);
                    }
                }
            }
        }
    }
}

#[test]
fn board_add_hazards() {
    for t in [
        Tile::Food,
        Tile::Snake,
        Tile::Head,
        Tile::Empty,
        Tile::Hazard(1),
        Tile::Hazard(2),
        Tile::Hazard(3),
        Tile::HazardWithFood(1),
        Tile::HazardWithFood(2),
        Tile::HazardWithFood(3),
        Tile::HazardWithSnake(1),
        Tile::HazardWithSnake(2),
        Tile::HazardWithSnake(3),
        Tile::HazardWithHead(1),
        Tile::HazardWithHead(2),
        Tile::HazardWithHead(3),
    ] {
        let mut b = Board::new(5, 5);
        assert_eq!(b.width(), 5);
        assert_eq!(b.height(), 5);
        for x in 0..5 {
            for y in 0..5 {
                let p = &Point { x, y };
                assert_eq!(b.get(p), Tile::Empty);
                b.set(p, t);
                let check = b.get(p);
                assert_eq!(
                    check, t,
                    "tried to set {} to {:?}, but read back {:?} - board = {:?}",
                    p, t, check, b
                );

                assert_eq!(
                    t.hazard_count(),
                    b.hazard_count(p),
                    "hazard count mismatch for {:?}",
                    t
                );

                for hazard_count in 0..MAX_HAZARDS {
                    let expected_value = cmp::min(MAX_HAZARDS, hazard_count + t.hazard_count());
                    let added_tile = Tile::Hazard(hazard_count);

                    b.add(p, added_tile);
                    let check = b.get(p);

                    assert_eq!(
                        t.has_food(),
                        check.has_food(),
                        "adding {:?} ({} hazards) to {:?} changed tile to {:?}",
                        added_tile,
                        hazard_count,
                        t,
                        check
                    );
                    assert_eq!(
                        t.is_snake(),
                        check.is_snake(),
                        "adding {:?} ({} hazards) to {:?} changed tile to {:?}",
                        added_tile,
                        hazard_count,
                        t,
                        check
                    );
                    assert_eq!(
                        expected_value,
                        check.hazard_count(),
                        "adding {:?} ({} hazards) to {:?} changed tile to {:?}",
                        added_tile,
                        hazard_count,
                        t,
                        check
                    );

                    while b.get(p) != t {
                        let before = b.get(p);
                        b.remove_hazards(p, 1);
                        let after = b.get(p);
                        assert_eq!(
                            before.hazard_count(),
                            after.hazard_count() + 1,
                            "failed to remove hazard: before {:?} vs after {:?}",
                            before,
                            after
                        )
                    }
                }

                for x in 0..5 {
                    for y in 0..5 {
                        let p2 = &Point { x, y };
                        if p != p2 {
                            let t2 = b.get(p2);
                            assert_eq!(
                                t2,
                                Tile::Empty,
                                "adding {} to {:?}, checking {} for damage: value is {:?} - board = {:?}",
                                p,
                                t,
                                p2,
                                t2,
                                b,
                            );
                        }
                    }
                }

                b.set(p, Tile::Empty);

                for x in 0..5 {
                    for y in 0..5 {
                        let p2 = &Point { x, y };
                        assert_eq!(b.get(p2), Tile::Empty);
                    }
                }
            }
        }
    }
}
