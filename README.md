# 🚀KTMU scrap🚀

#### 🚀 Blazingly fast 🚀, 🚀Memory-safe🚀, 🚀Optimized🚀 HTTP REST API server for 🚀schedule conversion🚀 from 🤮 https://ktmu-sutd.ru 🤮

### 🚀FAST🚀 Overview
  - **Getting daily or weekly schedule's JSON**
    1. `GET localhost:8080/schedule/daily` or `GET localhost:8080/schedule/weekly`
    2. Enojoy heavily nested and large JSON
    - Or, to get schedule only for one group, use something like `GET localhost:8080/schedule/daily?group=<GROUP>`
  - **Force update with a POST request**
    1. Get your temporary key if still didn't: `GET localhost:8080/schedule/interact`
    2. Request an update: `POST localhost:8080/schedule/update?key=<YOUR TEMP KEY>`
    3. After some time it'll return a JSON of changes in schedule (or `null` fields if there aren't any)
  - **Subscription to update events using WebSocket**
    1. Get your temporary key if still didn't: `GET localhost:8080/schedule/interact`
    2. WebSocket to `localhost:8080/schedule/updates?key=<YOUR TEMP KEY>`
    3. Periodically (10 min) it'll send a JSON of changes in schedule (or `null` fields if there aren't any)

### Why use inTeRAKtors and kEYs
To avoid duplicates 🤪

When you are attached to WebSocket events and also make POST update request, you may get the same notify as a **WebSocket event** AND as an **update response** 😮

So keys is just a filter to determine if a WebSocket client should receive the notify

### Where it's used
[**ktmuslave**](https://github.com/kerdl/ktmuslave) is a schedule bot for this server working both in VK and Telegram with some cool features

Probs not useful for anything else except for learning 🤔