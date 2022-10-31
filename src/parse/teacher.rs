use crate::REGEX;


pub fn extract_from_end(string: &mut String) -> Vec<String> {
    let mut teachers = vec![];

    // while we can find a teacher from THE END of the string
    while let Some(teacher) = REGEX.end_teacher.find(&string) {
        // push this teacher match to vec
        teachers.push(teacher.as_str().to_owned());

        // from end of the string, remove this teacher
        *string = REGEX.end_teacher.replace(string, "").to_string();

        // if there's another teacher left
        if REGEX.teacher.is_match(&string) {
            // remove last characters from the string
            // until it ends with something like `Д.`
            while REGEX.initial.find(&string).is_none() {
                string.pop();
            }
        }

        // remove whitespaces from the beginning and end
        *string = string.trim().to_string();
    }

    teachers.into_iter().rev().collect()
}