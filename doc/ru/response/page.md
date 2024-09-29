# При успехе

Определение: [`crate::api::Response`](/src/api/mod.rs?blame=1#L84)

Ссылки:
- [`Page`](/doc/ru/object/page.md)

```json
{
    "is_ok": true,
    "data": {
        "page": Page
    }
}
```


# При ошибке `NoLastSchedule`

Определение: [`crate::api::Response`](/src/api/mod.rs?blame=1#L84)

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
