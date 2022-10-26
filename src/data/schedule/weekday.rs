use strum_macros::EnumString;


#[derive(EnumString)]
pub enum LowercaseWeekday {
    #[strum(to_string = "понедельник")]
    Monday,
    #[strum(to_string = "вторник")]
    Tuesday,
    #[strum(to_string = "среда")]
    Wednesday,
    #[strum(to_string = "четверг")]
    Thursday,
    #[strum(to_string = "пятница")]
    Friday,
    #[strum(to_string = "суббота")]
    Saturday,
    #[strum(to_string = "воскресенье")]
    Sunday
}
