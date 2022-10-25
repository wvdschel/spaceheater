use crate::logic::{Board, Point, Tile};

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
