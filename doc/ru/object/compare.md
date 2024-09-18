# Compare

Определение: [`crate::compare::schedule::Page`](/src/compare/schedule.rs?blame=1#L172)

Ссылки:
- [`Formation`](/doc/ru/object/formation.md)
- [`Day`](/doc/ru/object/day.md)
- [`Subject`](/doc/ru/object/subject.md)
- [`Attender`](/doc/ru/object/attender.md)

```json
{
  "date": {
    "old": {
      "start": "YYYY-MM-DD",
      "end": "YYYY-MM-DD"
    } | null,
    "new": {
      "start": "YYYY-MM-DD",
      "end": "YYYY-MM-DD"
    } | null
  },
  "formations": {
    "appeared": [Formation],
    "disappeared": [Formation],
    "changed": [
      {
        "name": "<имя формирования>",
        "days": [
          {
            "appeared": [Day],
            "disappeared": [Day],
            "changed": [
              {
                "date": "YYYY-MM-DD",
                "subjects": {
                  "appeared": [Subject],
                  "disappeared": [Subject],
                  "changed": [
                    {
                      "name": "<имя предмета>",
                      "num": {
                        "old": uint32,
                        "new": uint32
                      } | null,
                      "attenders": [
                        {
                          "appeared": [Attender],
                          "disappeared": [Attender],
                          "changed": [
                            {
                              "name": "<имя посетителя>",
                              "cabinet": {
                                "primary": {
                                  "old": "<кабинет>" | null,
                                  "new": "<кабинет>" | null,
                                } | null,
                                "opposite": {
                                  "old": "<кабинет>" | null,
                                  "new": "<кабинет>" | null,
                                } | null
                              }
                            },
                            ...
                          ]
                        },
                        ...
                      ] | null
                    },
                    ...
                  ]
                }
              },
              ...
            ],
          },
          ...
        ]
      },
      ...
    ],
  }
}
```