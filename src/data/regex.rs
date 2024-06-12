use regex::Regex;
use std::sync::Arc;


pub struct Container {
    /// ## Matches
    /// - **"1КДД69"**
    /// - **"1-кДД-69"**
    /// - ...
    pub group: Arc<Regex>,
    /// ## Matches
    /// - **"1КДД69"**
    /// - **"1-кДД-69"**
    /// but from the start of the string
    pub start_group: Arc<Regex>,
    /// ## Matches
    /// - **"01.01.22"**
    /// - **"01/01/22"**
    /// - **"01.01.2022"**
    /// - **"01/01/2022"**
    /// - ...
    /// 
    /// ( *day*.*month*.*year* )
    pub date: Arc<Regex>,
    /// ## Matches
    /// - **"8:00-9:00"**
    /// - **"10:00-11:00"**
    /// - **"8:00–9:00"**
    /// - **"10:00–11:00"**
    /// - ...
    pub time: Arc<Regex>,
    /// ## Matches
    /// - **"8:00"**
    /// - **"09:00"**
    /// - **"10:00"**
    /// - ...
    pub single_time: Arc<Regex>,
    /// ## Matches
    /// - **"-"**
    /// - **"–"**
    pub time_sep: Arc<Regex>,
    /// ## Matches
    /// - **"Ебанько Х."**
    /// - **"Ебанько Х.Й"**
    /// - **"Ебанько Х.Й."**
    /// - ...
    pub teacher: Arc<Regex>,
    /// ## Matches
    /// - **"Ебанько Хуй Йебанько"**
    /// - ...
    pub teacher_full: Arc<Regex>,
    /// ## The same as `teacher`, but matches from the end
    pub end_teacher: Arc<Regex>,
    /// ## Matches teacher's initial from the end
    /// - Ебанько Х.`Й.`
    pub initial: Arc<Regex>,
    /// ## Matches
    /// - **"ауд.29"**
    /// - **"ауд.56,54"**
    /// - **"ауд.сп.з,23в"**
    /// - ...
    pub cabinet: Arc<Regex>,
    pub nonword: Arc<Regex>,
    pub digit: Arc<Regex>,
}
impl Default for Container {
    fn default() -> Container {
        let group = r"([0-9])([-]{0,1})([а-яёА-ЯЁ]{3})([-]{0,1})([0-9]{1,2})";
        let start_group = format!(r"^{}", group);
        let date = r"(\d{1,2})\W(\d{1,2})\W(\d{4}|\d{2})";
        let time = r"(\d{1,2}:\d{2})[-–](\d{1,2}:\d{2})";
        let single_time = r"(\d{1,2}:\d{2})";
        let time_sep = r"[-–]";
        let teacher = r"([А-ЯЁ][а-яё]{1,})(\s)([А-ЯЁ]{1}[.])([А-ЯЁ]{1}[.]{0,1}){0,1}";
        let teacher_full = r"([A-ZА-ЯЁ][a-za-яё]{1,}\s[A-ZА-ЯЁ][a-za-яё]{1,}\s[A-ZА-ЯЁ][a-za-яё]{1,})";
        let end_teacher = format!(r"({})$", teacher);
        let initial = r"([А-ЯЁ][.])$";
        let cabinet = r"([аa][уy]д)[.].+";
        let nonword = r"\W";
        let digit = r"\d";

        Container {
            group: Arc::new(Regex::new(group).unwrap()), 
            start_group: Arc::new(Regex::new(&start_group).unwrap()), 
            date: Arc::new(Regex::new(date).unwrap()), 
            time: Arc::new(Regex::new(time).unwrap()),
            single_time: Arc::new(Regex::new(single_time).unwrap()),
            time_sep: Arc::new(Regex::new(time_sep).unwrap()),
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