# Attedner

Definition: [`crate::data::schedule::Attender`](/src/data/schedule/mod.rs?blame=1#L85)

```json
{
  "raw": "<raw attender name>",
  "recovered": bool,
  "kind": "teacher" | "group",
  "name": "<attender name>",
  "cabinet": {
    "primary": "<cabinet>" | null,
    "opposite": "<cabinet>" | null
  }
}
```