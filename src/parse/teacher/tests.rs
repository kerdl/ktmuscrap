use super::*;

#[test]
fn test_validate() {
    let regex_own = crate::data::regex::Container::default();
    unsafe { crate::REGEX = &regex_own };

    assert_eq!(validate("Ебланов Х.Й."), Some("Ебланов Х.Й.".to_string()));
    assert_eq!(validate("Ебланов.Х.Й."), Some("Ебланов Х.Й.".to_string()));
    assert_eq!(validate("Ебланов Х. Й."), Some("Ебланов Х.Й.".to_string()));
    assert_eq!(validate("Ебланов.Х. Й."), Some("Ебланов Х.Й.".to_string()));
    assert_eq!(validate("Ебланов Х."), Some("Ебланов Х.".to_string()));
    assert_eq!(validate("Ебланов.Х."), Some("Ебланов Х.".to_string()));
    assert_eq!(validate("Ебланов"), None);
    assert_eq!(validate("Ебланов."), None);
    assert_eq!(validate(""), None);
}