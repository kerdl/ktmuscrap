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
- Atomatic download from Google Sheets
- Unpacking and HTML parsing
- Persistence of parsed data on disk
- Diff generation
- REST API with WebSocket update events


## Documentation
- [Running instructions](/doc/en/running.md)
- [Configuring](/doc/en/configuring.md)
- [API](/doc/en/api.md)
- [Code structure tour](/doc/en/tour.md)


## Where is it used
[**ktmuslave**](https://github.com/kerdl/ktmuslave) is a schedule bot built on top of this server. Working both in VK and Telegram.

Pointless for anything else 🤔
