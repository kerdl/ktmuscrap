use colors_transform::Color;
use crate::data;
use crate::data::schedule;
use super::*;


#[tokio::test]
async fn test_groups_1() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Учет страховых договоров Иванова А.А.";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Учет страховых договоров Иванова А.А.".to_string(),
        name: "Учет страховых договоров".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Иванова А.А.".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Иванова А.А.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_2() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Ин. язык Костина С.В. / Хачатрян Н.В.";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Ин. язык Костина С.В. / Хачатрян Н.В.".to_string(),
        name: "Ин. язык".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Костина С.В. / ".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Костина С.В.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "Хачатрян Н.В.".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Хачатрян Н.В.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_3() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Деловой англ. /Хачатрян Н.В.";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Деловой англ. /Хачатрян Н.В.".to_string(),
        name: "Деловой англ.".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Хачатрян Н.В.".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Хачатрян Н.В.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_4() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Деловой англ. Коняева А.С.37а/Хачатрян Н.В.";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Деловой англ. Коняева А.С.37а/Хачатрян Н.В.".to_string(),
        name: "Деловой англ.".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Коняева А.С.37а/".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Коняева А.С.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: Some("37а".to_string()),
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "Хачатрян Н.В.".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Хачатрян Н.В.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_5() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Деловой англ. Коняева А.С. /Хачатрян Н.В. каб 2";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Деловой англ. Коняева А.С. /Хачатрян Н.В. каб 2".to_string(),
        name: "Деловой англ.".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Коняева А.С. /".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Коняева А.С.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "Хачатрян Н.В. каб 2".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Хачатрян Н.В.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: Some("каб 2".to_string()),
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_6() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Деловой англ. Коняева А.С. 37а/Хачатрян Н.В. каб 2";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Деловой англ. Коняева А.С. 37а/Хачатрян Н.В. каб 2".to_string(),
        name: "Деловой англ.".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Коняева А.С. 37а/".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Коняева А.С.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: Some("37а".to_string()),
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "Хачатрян Н.В. каб 2".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Хачатрян Н.В.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: Some("каб 2".to_string()),
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_7() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Экономика организации Вакансия 05";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Экономика организации Вакансия 05".to_string(),
        name: "Экономика организации".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Вакансия 05".to_string(),
                kind: schedule::attender::Kind::Vacancy,
                name: "Вакансия 05".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_groups_8() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "Выполнение дизайнерских проектов в материале Вакансия 02.3/Натус Н.И. каб. 4";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "Выполнение дизайнерских проектов в материале Вакансия 02.3/Натус Н.И. каб. 4".to_string(),
        name: "Выполнение дизайнерских проектов в материале".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "Вакансия 02.3/".to_string(),
                kind: schedule::attender::Kind::Vacancy,
                name: "Вакансия 02.3".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "Натус Н.И. каб. 4".to_string(),
                kind: schedule::attender::Kind::Teacher,
                name: "Натус Н.И.".to_string(),
                cabinet: schedule::Cabinet {
                    primary: Some("каб. 4".to_string()),
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(groups(string, num, color), result);
}

#[tokio::test]
async fn test_teachers_1() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "3дд48 жив каб 17а";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "3дд48 жив каб 17а".to_string(),
        name: "жив".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "3дд48".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "3КДД48".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: Some("каб 17а".to_string()),
            opposite: None
        }
    };
    assert_eq!(teachers(string, num, color), result);
}

#[tokio::test]
async fn test_teachers_2() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "3рд33/4рд34/36 осн фил";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "3рд33/4рд34/36 осн фил".to_string(),
        name: "осн фил".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "3рд33".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "3КРД33".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "4рд34".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "4КРД34".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "36".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "4КРД36".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(teachers(string, num, color), result);
}

#[tokio::test]
async fn test_teachers_3() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "1 мп2\\4 лит-ра";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "1 мп2\\4 лит-ра".to_string(),
        name: "лит-ра".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "1 мп2".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "1КМП2".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            },
            schedule::Attender {
                raw: "4".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "1КМП4".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: None,
            opposite: None
        }
    };
    assert_eq!(teachers(string, num, color), result);
}

#[tokio::test]
async fn test_teachers_4() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "1ктд4   ОБЗР каб.40";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "1ктд4   ОБЗР каб.40".to_string(),
        name: "ОБЗР".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "1ктд4".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "1КТД4".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: Some("каб.40".to_string()),
            opposite: None
        }
    };
    assert_eq!(teachers(string, num, color), result);
}

#[tokio::test]
async fn test_teachers_5() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "2рд36 культ каб ?";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "2рд36 культ каб ?".to_string(),
        name: "культ".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "2рд36".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "2КРД36".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: Some("каб ?".to_string()),
            opposite: None
        }
    };
    assert_eq!(teachers(string, num, color), result);
}

#[tokio::test]
async fn test_teachers_6() {
    let data_path = [".", "data"].iter().collect();
    let regex_own = data::regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
    unsafe {
        crate::REGEX = &regex_own;
        crate::DATA = &data_own;
    }

    let string = "1дд20 ПС актовый за";
    let num = 2;
    let color = "#c6d9f0"
        .parse::<colors_transform::Rgb>()
        .map(|color| palette::Srgb::new(
            color.get_red(),
            color.get_green(),
            color.get_blue()
        ))
        .unwrap();
    let result = schedule::Subject {
        raw: "1дд20 ПС актовый за".to_string(),
        name: "ПС".to_string(),
        num: 2,
        format: schedule::raw::Format::Remote,
        attenders: vec![
            schedule::Attender {
                raw: "1дд20".to_string(),
                kind: schedule::attender::Kind::Group,
                name: "1КДД20".to_string(),
                cabinet: schedule::Cabinet {
                    primary: None,
                    opposite: None
                }
            }
        ],
        cabinet: schedule::Cabinet {
            primary: Some("актовый за".to_string()),
            opposite: None
        }
    };
    assert_eq!(teachers(string, num, color), result);
}