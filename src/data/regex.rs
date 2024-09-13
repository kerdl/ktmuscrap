use regex::Regex;
use std::sync::Arc;


pub struct Container {
    /// ## Match examples
    /// - 4-КРД-36
    /// - 4-крд-36
    /// - 4КРД36
    /// - 4крд36
    /// - 4-КРД-3
    /// - 4-крд-3
    /// - 4КРД3
    /// - 4крд3
    /// - 4-РД-36
    /// - 4-рд-36
    /// - 4РД36
    /// - 4рд36
    /// - 4 РД36
    /// - 4 рд36
    /// - 4РД 36
    /// - 4рд 36
    /// - 4 РД 36
    /// - 4 рд 36
    /// - 4-РД-3
    /// - 4-рд-3
    /// - 4РД3
    /// - 4рд3
    /// - 4 РД3
    /// - 4 рд3
    /// - 4РД 3
    /// - 4рд 3
    /// - 4 РД 3
    /// - 4 рд 3
    pub group: Arc<Regex>,
    /// Same as `group` but asserts the start of the string
    pub start_group: Arc<Regex>,
    /// ## Match examples
    /// - `пн`
    /// - `вТ`0
    /// - -0`СР`
    /// ## Doesn't match
    /// - Пнф
    /// - свт
    pub whole_short_weekday: Arc<Regex>,
    /// ## Match examples
    /// - 01.01
    /// - 01/01
    /// - 01.01.22
    /// - 01/01/22
    /// - 01.01.2022
    /// - 01/01/2022
    /// 
    /// (*day*.*month*)
    /// (*day*.*month*.*year*)
    pub date: Arc<Regex>,
    /// ## Matches
    /// - Ебанько Х.
    /// - Ебанько Х.Й
    /// - Ебанько Х.Й.
    /// - ...
    pub teacher: Arc<Regex>,
    /// ## Matches
    /// - Ебанько Хуй Йебанько
    /// - ...
    pub teacher_full: Arc<Regex>,
    /// ## Same as `teacher` but matches from the end
    pub end_teacher: Arc<Regex>,
    /// ## Matches teacher's initial from the end
    /// - Ебанько Х.`Й.`
    pub initial: Arc<Regex>,
    /// ## Matches
    /// - ауд.29
    /// - ауд.56,54
    /// - ауд.сп.з,23в
    /// - ...
    pub cabinet: Arc<Regex>,
    pub nonword: Arc<Regex>,
    pub digit: Arc<Regex>,
}
impl Default for Container {
    fn default() -> Container {
        let group = r"([0-9])([-]*|\s*)([а-яёА-ЯЁ]{2,3})([-]*|\s*)([0-9]{1,2})";
        let start_group = format!(r"^{}", group);
        let whole_short_weekday = r"(?<![а-яёА-ЯЁ])([пП][нН]|[вВ][тТ]|[сС][рР]|[чЧ][тТ]|[пП][тТ]|[сС][бБ]|[вВ][сС])(?![а-яёА-ЯЁ])";
        let date = r"(\d{1,2})\W(\d{1,2})(\W(\d{4}|\d{2}))*";
        let teacher = r"([А-ЯЁ][а-яё]{1,})(\s)([А-ЯЁ]{1}[.])([А-ЯЁ]{1}[.]?)?";
        let teacher_full = r"([A-ZА-ЯЁ][a-za-яё]+\s[A-ZА-ЯЁ][a-za-яё]+\s[A-ZА-ЯЁ][a-za-яё]+)";
        let end_teacher = format!(r"({})$", teacher);
        let initial = r"([А-ЯЁ][.])$";
        let cabinet = r"([аa][уy]д)[.].+";
        let nonword = r"\W";
        let digit = r"\d";

        Container {
            group: Arc::new(Regex::new(group).unwrap()), 
            start_group: Arc::new(Regex::new(&start_group).unwrap()), 
            whole_short_weekday: Arc::new(Regex::new(whole_short_weekday).unwrap()), 
            date: Arc::new(Regex::new(date).unwrap()), 
            teacher: Arc::new(Regex::new(teacher).unwrap()),
            teacher_full: Arc::new(Regex::new(teacher_full).unwrap()),
            end_teacher: Arc::new(Regex::new(&end_teacher).unwrap()), 
            initial: Arc::new(Regex::new(initial).unwrap()), 
            cabinet: Arc::new(Regex::new(cabinet).unwrap()),
            nonword: Arc::new(Regex::new(nonword).unwrap()),
            digit: Arc::new(Regex::new(digit).unwrap()),
        }
    }
}