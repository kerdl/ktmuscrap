use super::*;

#[test]
fn test_validate() {
    let regex_own = crate::data::regex::Container::default();
    unsafe { crate::REGEX = &regex_own };

    assert_eq!(validate("3кдд48"), Some("3КДД48".to_string()));
    assert_eq!(validate("3-кдд-48"), Some("3КДД48".to_string()));
    assert_eq!(validate("3дд48"), Some("3КДД48".to_string()));
    assert_eq!(validate("3-дд-48"), Some("3КДД48".to_string()));
    assert_eq!(validate("1крд2"), Some("1КРД2".to_string()));
    assert_eq!(validate("1-крд-2"), Some("1КРД2".to_string()));
    assert_eq!(validate("1рд2"), Some("1КРД2".to_string()));
    assert_eq!(validate("1-рд-2"), Some("1КРД2".to_string()));
    assert_eq!(validate("жив каб 17а"), None);
}

#[test]
fn test_multi() {
    let regex_own = crate::data::regex::Container::default();
    unsafe { crate::REGEX = &regex_own };

    assert_eq!(
        multi("3дд48 жив каб 17а"),
        Some(vec![
            (0 as usize..7 as usize, "3КДД48".to_string())
        ])
    );
    assert_eq!(
        multi("1крд2/4/6 истор"),
        Some(vec![
            (0 as usize..8 as usize, "1КРД2".to_string()),
            (9 as usize..10 as usize, "1КРД4".to_string()),
            (11 as usize..12 as usize, "1КРД6".to_string())
        ])
    );
    assert_eq!(
        multi("3рд33/4рд34/36 осн фил"),
        Some(vec![
            (0 as usize..7 as usize, "3КРД33".to_string()),
            (8 as usize..15 as usize, "4КРД34".to_string()),
            (16 as usize..18 as usize, "4КРД36".to_string())
        ])
    );
}
