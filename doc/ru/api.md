# API
Все ответы в JSON.


### Получение расписания групп → [Page](/doc/ru/response/page.md)
```
GET http://localhost:8080/schedule/groups
```
Расписание для всех представленных групп.


### Получение расписания для определённой группы → [Page](/doc/ru/response/page.md)
```
GET http://localhost:8080/schedule/groups?name=<точное имя группы>
```
Расписание только для указанной группы.


### Получение расписания преподавателей → [Page](/doc/ru/response/page.md)
```
GET http://localhost:8080/schedule/teachers
```
Расписание для всех представленных преподавателей.


### Получение расписания для определённого преподавателя → [Page](/doc/ru/response/page.md)
```
GET http://localhost:8080/schedule/teachers?name=<точное имя препода>
```
Расписание только для указанного преподавателя.


### Подключение WebSocket с обновлениями → [Notify](/doc/ru/object/notify.md)
```
WS ws://localhost:8080/schedule/updates
```
Канал обновлений с изменениями в расписании.

Каждый раз после обновления расписаний,
ktmuscrap ищет изменения и рассылает
их каждому подключившемуся.
Разница (diff) присылается
всегда, независимо от того,
есть ли изменения или нет.


### Получение времени последнего обновления → [Updates](/doc/ru/response/updates.md)
```
GET http://localhost:8080/schedule/updates/last
```
Когда было произведено последнее обновление.


### Получение периода обновления → [Updates](/doc/ru/response/updates.md)
```
GET http://localhost:8080/schedule/updates/period
```

Как часто производятся обновления.
Это значение устанавливается в конфиге.