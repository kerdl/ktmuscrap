[🇷🇺 Русский](/README-RU.md)

# Schedule parser from [ktmu-sutd.ru](https://ktmu-sutd.ru)

Funny little note:
- I acknowledge the shittiness of this code
- Couldn't fucking care less
- No one pays me for that
- This is the last version,
no further updates in case of changing schedule formats
- L + Ratio


## Overview
This is a HTTP REST API server with a schedule parser under the hood.

- Every 10 minutes (configurable) ZIP archives
from specified URLs to Google Sheets are downloaded
- ZIPs are extracted and HTMLs are parsed
- Parsed data is being saved onto the disk,
including the time of last update
- Schedule diffs are generated and sent
to the connected WebSocket clients
- Client uses server APIs to view freshly parsed schedules


## API
All responses are in JSON.


### Getting groups schedule → [Page](/doc/en/response/page.md)
```
GET http://localhost:8080/schedule/groups
```
A schedule for all the groups present.


### Getting schedule for specific group → [Page](/doc/en/response/page.md)
```
GET http://localhost:8080/schedule/groups?name=<exact group name>
```
A schedule containing only the specified group.


### Getting teachers schedule → [Page](/doc/en/response/page.md)
```
GET http://localhost:8080/schedule/teachers
```
A schedule for all the teachers present.


### Getting schedule for specific teacher → [Page](/doc/en/response/page.md)
```
GET http://localhost:8080/schedule/teachers?name=<exact teacher name>
```
A schedule containing only the specified teacher.


### WebSocket connection with updates → [Notify](/doc/en/object/notify.md)
```
WS ws://localhost:8080/schedule/updates
```
An update channel with diffs.

Every time the schedule updates,
ktmuscrap generates a diff and sends it
to everyone connected.
This diff is always sent, no matter
the changes - if there any or not.


### Getting last update time → [Updates](/doc/en/response/updates.md)
```
GET http://localhost:8080/schedule/updates/last
```
When was the last update performed.


### Getting update period → [Updates](/doc/en/response/updates.md)
```
GET http://localhost:8080/schedule/updates/period
```
How often updates are performed. This value is set in the config.


## Where is it used
[**ktmuslave**](https://github.com/kerdl/ktmuslave) is a schedule bot built on top of this server. Working both in VK and Telegram.

Pointless for anything else 🤔
