# Configuring
**ktmuscrap** uses 2 configuration files.


## Contents
- [Settings](#settings)
- [Schedules](#schedules)
- [Schedules example](#schedules-example)


## Settings
File: `./data/settings.json`
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
The address on which the API server will be running.

### `parsing.fulltime_color`
Fulltime subject hex color for classification.

### `parsing.remote_color`
Remote subject hex color for classification.


## Schedules
File: `./data/schedule/index.json`
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
Should it fetch the schedules.

### `updated`
Last update time.

### `period`
Schedule update period.

### `ignored`
List of schedule names ignored during download. 

**Example**:
```json
"ignored": ["groups-4", "teachers"]
```

### `types`
Schedule descriptions: their names, download URLs
and timeouts.

**Object**:
```json
{
  "kind": "groups | teachers",
  "name": "schedule_name",
  "url": "https://docs.google.com/spreadsheets/d/abcdef/preview",
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
Specifies the schedule kind.
```
"groups" | "teachers"
```

#### `name`
Any name to this schedule.

Used
- as a folder name during unpacking
- for identification in [`ignored`](#ignored)

#### `url`
URL to a ZIP archive with the schedule.

If the URL points to Google Sheets, make sure it
has `/preview` at the end.

Support for `/export?format=zip` is removed,
because the editor could disable exporting.

#### `fetch_timeout`
Maximum allowed time to download this schedule.
Exceeding it will cause it cancel and starting over.

#### `retry_period`
Waiting time between downloading tries.

**Example**:
```json
"types": [
  {
    {
      "kind": "groups",
      "name": "groups-1",
      "url": "https://docs.google.com/spreadsheets/d/abcdef/preview",
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
      "url": "https://docs.google.com/spreadsheets/d/ghijkl/preview",
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


## Schedules example
File: `./data/schedule/index.json`
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
      "kind": "groups",
      "name": "groups-1",
      "url": "https://docs.google.com/spreadsheets/d/abcdef/preview",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    },
    {
      "kind": "groups",
      "name": "groups-2",
      "url": "https://docs.google.com/spreadsheets/d/ghijkl/preview",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    },
    {
      "kind": "groups",
      "name": "groups-3",
      "url": "https://docs.google.com/spreadsheets/d/mnopqr/preview",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    },
    {
      "kind": "groups",
      "name": "groups-4",
      "url": "https://docs.google.com/spreadsheets/d/stuvwx/preview",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    },
    {
      "kind": "teachers",
      "name": "teachers",
      "url": "https://docs.google.com/spreadsheets/d/yzabcd/preview",
      "fetch_timeout": {
        "secs": 90,
        "nanos": 0
      },
      "retry_period": {
        "secs": 2,
        "nanos": 0
      }
    }
  ]
}
```