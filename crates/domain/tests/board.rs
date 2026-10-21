#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use starter_domain::board::{Board, MAX_TASKS, Title};
#[test]
fn title_boundaries_and_task_capacity_are_explicit() {
    for bad in ["", "  ", "a\nb", "a\tb"] {
        assert!(Title::try_from(bad).is_err());
    }
    assert!(Title::try_from("é".repeat(120).as_str()).is_ok());
    assert!(Title::try_from("é".repeat(121).as_str()).is_err());
    assert_eq!(Title::try_from(" Zoë ").unwrap().as_str(), "Zoë");
    let mut board = Board::default();
    for _ in 0..MAX_TASKS {
        board.add(Title::try_from("task").unwrap()).unwrap();
    }
    let before = board.clone();
    assert!(
        board
            .add(Title::try_from("over capacity").unwrap())
            .is_err()
    );
    assert_eq!(board, before);
    assert!(board.remove(999).is_err());
    assert!(board.complete(999).is_err());
    assert_eq!(board, before);
}
