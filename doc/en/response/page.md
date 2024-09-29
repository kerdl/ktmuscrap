# On success

Definition: [`crate::api::Response`](/src/api/mod.rs?blame=1#L84)

References:
- [`Page`](/doc/en/object/page.md)

```json
{
  "is_ok": true,
  "data": {
    "page": Page
  }
}
```


# On error `NoLastSchedule`

Definition: [`crate::api::Response`](/src/api/mod.rs?blame=1#L84)

```json
{
  "is_ok": false,
  "error": {
    "kind": "internal_failure",
    "error": "NoLastSchedule",
    "text": "no schedule found, make sure tables are still available and are valid"
  }
}
```
