# Запуск
**ktmuscrap** работает на Windows и Linux.

Доступна виртуализация через Docker.


## Содержание
- [Docker под Linux](#docker-под-linux)
- [Windows](#windows)
- [Debian Linux](#debian-linux)


## Docker под Linux
Ты можешь запустить контейнер **ktmuslave** сразу со всеми нужными зависимостями.

Инструкция здесь.


## Windows
1. Убедись, что установлен [.NET Framework](https://support.microsoft.com/en-us/topic/microsoft-net-framework-4-8-offline-installer-for-windows-9d23f658-3b97-68ab-d013-aa3c3e7495e0)
версии не ниже 4.6
2. Установи [stable Rust](https://www.rust-lang.org/tools/install)
стандартным способом
3. Скачай и распакуй репозиторий,
либо воспользуйся **git**:
```bash
git clone https://github.com/kerdl/ktmuscrap
```
4. Перейди в директорию с кодом или открой там **cmd**:
```bash
cd ktmuscrap-yr2024
```
5. Запусти и дождись сообщения
```
cargo run --release
...
0000-00-00 at 00:00:00 [ERROR] before running, see ./data/schedule/index.json and fill in the schedule types manually (src/main.rs:55)
```
6. Открой файл по пути `./data/schedule/index.json` и заполни поле `types`
по [этой документации](/doc/ru/configuring.md#types)
7. Запусти снова
```bash
cargo run --release
```


## Debian Linux
1. Установи зависимости
```bash
sudo apt update && sudo apt install git build-essential pkg-config libssl-dev -y
```
2. Установи stable Rust стандартным способом
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. Обнови переменные среды
```bash
source $HOME/.cargo/env
```
4. Склонируй репозиторий
```bash
git clone https://github.com/kerdl/ktmuscrap
```
5. Перейди в директорию
```bash
cd ktmuscrap-yr2024
```
6. Запусти и дождись сообщения
```
cargo run --release
...
0000-00-00 at 00:00:00 [ERROR] before running, see ./data/schedule/index.json and fill in the schedule types manually (src/main.rs:55)
```
7. Открой файл по пути `./data/schedule/index.json` и заполни поле `types`
по [этой документации](/doc/ru/configuring.md#types)
```bash
nano data/schedule/index.json
```
8. Запусти снова
```bash
cargo run --release
```
