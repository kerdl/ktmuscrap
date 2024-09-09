# Schedule parser from https://ktmu-sutd.ru

#### Funny little note
- I acknowledge the shittiness of this code
- Couldn't fucking care less
- No one pays me for that
- This is the last version,
no further adaptations in case of changing schedule formats
- L + Ratio

## Overview
This is a regular HTTP REST API server with a schedule parser under the hood.

- Every 10 minutes (configurable) ZIP archives
from specified URLs to Google Sheets are downloaded
- ZIPs are extracted and HTMLs are parsed
- Parsed data is being saved on the disk,
including the time of last update
- Schedule diffs are generated and sent
to the connected WebSocket clients
- Client uses server APIs to view freshly parsed schedules

## API
All responses are in JSONs.
Schemas and examples later.

### Getting groups schedule
`GET http://localhost:8080/schedule/groups`

A schedule for all the groups presented.

### Getting schedule for specific group
`GET http://localhost:8080/schedule/groups?name=<EXACT GROUP NAME>`

A schedule containing only the specified group.

### Getting teachers schedule
`GET http://localhost:8080/schedule/teachers`

A schedule for all the teachers presented.

### Getting schedule for specific teacher
`GET http://localhost:8080/schedule/teachers?name=<EXACT TEACHER NAME>`

A schedule containing only the specified teacher.

### Websocket connection with updates
`WS ws://localhost:8080/schedule/updates`

An update channel with diffs.

Every time after schedule update,
ktmuscrap generates a diff and sends it
to everyone connected.
This diff is always sent, no matter
the changes - if there any or not.

### Getting update period
`GET http://localhost:8080/schedule/updates/period`

How often updates are performed. This value is set in the config.

### Getting last update time
`GET http://localhost:8080/schedule/updates/period`

When was the last update performed.

## Where is it used
[**ktmuslave**](https://github.com/kerdl/ktmuslave) is a schedule bot built on top of this server. Working both in VK and Telegram.

Pointless for anything else 🤔