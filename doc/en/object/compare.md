# Compare

Definition: [`crate::compare::schedule::Page`](/src/compare/schedule.rs?blame=1#L172)

References:
- [`Formation`](/doc/en/object/formation.md)
- [`Day`](/doc/en/object/day.md)
- [`Subject`](/doc/en/object/subject.md)
- [`Attender`](/doc/en/object/attender.md)

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
        "name": "<formation name>",
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
                      "name": "<subject name>",
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
                              "name": "<attender name>",
                              "cabinet": {
                                "primary": {
                                  "old": "<cabinet>" | null,
                                  "new": "<cabinet>" | null,
                                } | null,
                                "opposite": {
                                  "old": "<cabinet>" | null,
                                  "new": "<cabinet>" | null,
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