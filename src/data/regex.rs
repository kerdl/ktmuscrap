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
    /// - ...
    pub group: Arc<Regex>,
    /// Same as `group` but asserts the start of the string
    pub start_group: Arc<Regex>,
    /// ## Match examples
    /// - `/`1дд69/37
    /// - `\`1дд69\37
    /// - ...
    pub start_attender_sep: Arc<Regex>,
    /// ## Match examples
    /// - 1дд69/37`/`
    /// - 1дд69\37`\`
    /// - ...
    pub end_attender_sep: Arc<Regex>,
    /// ## Match examples
    /// - `пн`
    /// - `вТ`0
    /// - -0`СР`
    /// - ...
    /// ## Doesn't match
    /// - Пнф
    /// - свт
    /// - ...
    pub whole_short_weekday: Arc<Regex>,
    /// ## Match examples
    /// - 01.01
    /// - 01/01
    /// - 01.01.22
    /// - 01/01/22
    /// - 01.01.2022
    /// - 01/01/2022
    /// - ...
    /// 
    /// (*day*.*month*)
    /// (*day*.*month*.*year*)
    pub date: Arc<Regex>,
    /// ## Matches
    /// - Ебанько Х.
    /// - Ебанько Х.Й
    /// - Ебанько Х.Й.
    /// - Ебанько.Х.Й.
    /// - ...
    pub teacher: Arc<Regex>,
    pub vacancy: Arc<Regex>,
    pub end_cabinet: Arc<Regex>,
    pub nonword: Arc<Regex>,
    pub digit: Arc<Regex>,
    pub start_digits: Arc<Regex>,
    pub end_digits: Arc<Regex>
}
impl Default for Container {
    fn default() -> Container {
        let group = r"([0-9])([-.]|\s)*([а-яёА-ЯЁ]{2,3})([-.]|\s)*([0-9]{1,2})";
        let start_group = format!(r"^{}", group);
        let start_attender_sep = r"^\s*[/\\]\s*";
        let end_attender_sep = r"(\s*[/\\]\s*)+$";
        let whole_short_weekday = r"\b([пП][нН]|[вВ][тТ]|[сС][рР]|[чЧ][тТ]|[пП][тТ]|[сС][бБ]|[вВ][сС])\b";
        let date = r"(\d{1,2})\W(\d{1,2})(\W(\d{4}|\d{2}))*";
        let teacher = r"([А-ЯЁ][а-яё]{1,})([^а-яёА-ЯЁa-zA-Z0-9_])([А-ЯЁ]{1}[.])\s*([А-ЯЁ]{1}[.]?)?";
        let vacancy = r"([А-ЯЁ][а-яё]{5,9})([^а-яёА-ЯЁa-zA-Z0-9_])(\d{1,3})([^а-яёА-ЯЁa-zA-Z0-9_]+\d+)?";
        let end_cabinet = r"(((([кКK][аАaA][бБ])[^а-яёА-ЯЁa-zA-Z0-9_]*)?([\d?]{1,3})[а-яёА-ЯЁa-zA-Z]*)|((([сСcC][пП][оОoO][рРpP][тТ])|([аАaA][кК][тТ].*))([^а-яёА-ЯЁa-zA-Z0-9_]){0,3}[зЗ][аАaA][лЛ]?))+$";
        let nonword = r"[^а-яёА-ЯЁa-zA-Z0-9_]";
        let digit = r"\d";
        let start_digits = r"^\d+";
        let end_digits = r"\d+$";

        Container {
            group: Arc::new(Regex::new(group).unwrap()), 
            start_group: Arc::new(Regex::new(&start_group).unwrap()),
            start_attender_sep: Arc::new(Regex::new(start_attender_sep).unwrap()),
            end_attender_sep: Arc::new(Regex::new(end_attender_sep).unwrap()),
            whole_short_weekday: Arc::new(Regex::new(whole_short_weekday).unwrap()), 
            date: Arc::new(Regex::new(date).unwrap()), 
            teacher: Arc::new(Regex::new(teacher).unwrap()),
            vacancy: Arc::new(Regex::new(vacancy).unwrap()),
            end_cabinet: Arc::new(Regex::new(end_cabinet).unwrap()),
            nonword: Arc::new(Regex::new(nonword).unwrap()),
            digit: Arc::new(Regex::new(digit).unwrap()),
            start_digits: Arc::new(Regex::new(start_digits).unwrap()),
            end_digits: Arc::new(Regex::new(end_digits).unwrap())
        }
    }
}