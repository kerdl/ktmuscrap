# API
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