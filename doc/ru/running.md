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
(требуется для установщика Rust)
2. Установи [stable Rust](https://www.rust-lang.org/tools/install)
стандартным способом
3. Скачай и распакуй репозиторий,
либо воспользуйся **git**:
```console
git clone https://github.com/kerdl/ktmuscrap
```
4. Перейди в директорию с кодом или открой там **cmd**:
```console
cd ktmuscrap-yr2024
```
или
```console
cd ktmuscrap
```
5. Создай необходимые папки и файл `index.json`
```console
mkdir data\schedule
copy NUL data\schedule\index.json
```
6. Заполни файл `index.json` по
[примеру](/doc/ru/configuring.md#пример-расписаний)
([документация](/doc/ru/configuring.md#расписания))
8. Запусти
```console
cargo run --release
```


## Debian Linux
1. Установи зависимости
```console
sudo apt update && sudo apt install git build-essential pkg-config libssl-dev -y
```
2. Установи stable Rust стандартным способом
```console
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. Обнови переменные среды
```console
source $HOME/.cargo/env
```
4. Склонируй репозиторий
```console
git clone https://github.com/kerdl/ktmuscrap
```
5. Перейди в директорию
```console
cd ktmuscrap
```
6. Создай необходимые папки и файл `index.json`
```console
mkdir -p data/schedule
touch data/schedule/index.json
```
7. Заполни файл `index.json` по
[примеру](/doc/ru/configuring.md#пример-расписаний)
([документация](/doc/ru/configuring.md#расписания))
```console
nano data/schedule/index.json
```
8. Запусти
```console
cargo run --release
```
