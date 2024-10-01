# Конфигурация
**ktmuscrap** использует 2 файла конфигурации.


## Содержание
- [Настройки](#настройки)
- [Расписания](#расписания)
- [Пример расписаний](#пример-расписания)


## Настройки
Файл: `./data/settings.json`
```json
{
  "server": {
    "address": "0.0.0.0:8080"
  },
  "parsing": {
    "fulltime_color": "#fce5cd",
    "remote_color": "#c6d9f0"
  }
}
```

### `server.address`
На каком адресе будет запущен API сервер.

### `parsing.fulltime_color`
Hex-цвет очного предмета для классификации.

### `parsing.remote_color`
Hex-цвет дистанционного предмета для классификации.


## Расписания
Файл: `./data/schedule/index.json`
```json
{
  "fetch": true,
  "updated": "1970-01-01T00:00:00",
  "period": {
    "secs": 600,
    "nanos": 0
  },
  "ignored": [],
  "types": []
}
```

### `fetch`
Скачивать ли расписания.

### `updated`
Время последнего обновления.

### `period`
Период регулярного обновления.

### `ignored`
Список имён расписаний, игнорируемые при скачивании.

**Пример**:
```json
"ignored": ["groups-4", "teachers"]
```

### `types`
Описание расписаний: их названия, ссылки на скачивание и тайм-ауты.

**Объект**:
```json
{
  "kind": "groups | teachers",
  "name": "schedule_name",
  "url": "https://docs.google.com/spreadsheets/d/abcdef/export?format=zip",
  "fetch_timeout": {
    "secs": 90,
    "nanos": 0
  },
  "retry_period": {
    "secs": 2,
    "nanos": 0
  }
}
```

#### `kind`
Указывает тип расписания.
```
"groups" | "teachers"
```

#### `name`
Произвольное имя этому расписанию.

Используется
- как имя папки при распаковке
- для идентификации в [`ignored`](#ignored)

#### `url`
Ссылка на ZIP-архив с расписанием.

Если ссылка на Google Таблицы, нужно убедиться в наличии `/export?format=zip` на конце.

#### `fetch_timeout`
Максимально разрешённое время на скачивание этого расписания.

После превышения скачивание начнётся заново.

#### `retry_period`
Время ожидания между повторными попытками скачивания.

**Пример**:
```json
"types": [
  {
    {
      "kind": "groups",
      "name": "groups-1",
      "url": "https://docs.google.com/spreadsheets/d/abcdef/export?format=zip",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    }
  },
  {
    {
      "kind": "teachers",
      "name": "teachers",
      "url": "https://docs.google.com/spreadsheets/d/ghijkl/export?format=zip",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    }
  }
]
```


## Пример расписаний
```json
{
  "fetch": true,
  "updated": "1970-01-01T00:00:00",
  "period": {
    "secs": 600,
    "nanos": 0
  },
  "ignored": [],
  "types": [
    {
      {
        "kind": "groups",
        "name": "groups-1",
        "url": "https://docs.google.com/spreadsheets/d/abcdef/export?format=zip",
        "fetch_timeout": {
          "secs": 90,
          "nanos": 0
        },
        "retry_period": {
          "secs": 2,
          "nanos": 0
        }
      }
    },
        {
      {
        "kind": "groups",
        "name": "groups-2",
        "url": "https://docs.google.com/spreadsheets/d/ghijkl/export?format=zip",
        "fetch_timeout": {
          "secs": 90,
          "nanos": 0
        },
        "retry_period": {
          "secs": 2,
          "nanos": 0
        }
      }
    },
        {
      {
        "kind": "groups",
        "name": "groups-3",
        "url": "https://docs.google.com/spreadsheets/d/mnopqr/export?format=zip",
        "fetch_timeout": {
          "secs": 90,
          "nanos": 0
        },
        "retry_period": {
          "secs": 2,
          "nanos": 0
        }
      }
    },
        {
      {
        "kind": "groups",
        "name": "groups-4",
        "url": "https://docs.google.com/spreadsheets/d/stuvwx/export?format=zip",
        "fetch_timeout": {
          "secs": 90,
          "nanos": 0
        },
        "retry_period": {
          "secs": 2,
          "nanos": 0
        }
      }
    },
    {
      {
        "kind": "teachers",
        "name": "teachers",
        "url": "https://docs.google.com/spreadsheets/d/yzabcd/export?format=zip",
        "fetch_timeout": {
          "secs": 90,
          "nanos": 0
        },
        "retry_period": {
          "secs": 2,
          "nanos": 0
        }
      }
    }
  ]
}
```