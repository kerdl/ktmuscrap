# Running
**ktmuscrap** works on Windows and Linux.

Virtualization through Docker is available.


## Contents
- [Docker on Debian-based Linux](#docker-on-debian-based-linux)
- [Windows](#windows)
- [Debian-based Linux](#debian-based-linux)


## Docker on Debian-based Linux
You can run a **ktmuslave** container with
all the dependencies included.

Instructions
[here](https://github.com/kerdl/ktmuslave/blob/yr2024/doc/en/running.md#docker-on-debian-based-linux).


## Windows
1. Make sure that the [.NET Framework](https://support.microsoft.com/en-us/topic/microsoft-net-framework-4-8-offline-installer-for-windows-9d23f658-3b97-68ab-d013-aa3c3e7495e0)
is installed and is not below 4.6
(required by Rust installer)
2. Install [stable Rust](https://www.rust-lang.org/tools/install)
using quick install
3. Download and unpack the repo,
or use **git**
```console
git clone https://github.com/kerdl/ktmuscrap
```
4. Go to the code directory or open a **cmd** there
```console
cd ktmuscrap-yr2024
```
or
```console
cd ktmuscrap
```
5. Create necessary folders and the `index.json` file
```console
mkdir data\schedule
copy NUL data\schedule\index.json
```
6. Fill in `index.json`
([example](/doc/en/configuring.md#schedules-example),
[documentation](/doc/en/configuring.md#schedules))
7. Run
```console
cargo run --release
```


## Debian-based Linux
1. Install dependencies
```console
sudo apt update && sudo apt install git build-essential pkg-config libssl-dev -y
```
2. Install stable Rust using quick install
```console
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. Update environment variables
```console
source $HOME/.cargo/env
```
4. Clone the repo
```console
git clone https://github.com/kerdl/ktmuscrap
```
5. Go to the directory
```console
cd ktmuscrap
```
6. Create necessary folders and the `index.json` file
```console
mkdir -p data/schedule
touch data/schedule/index.json
```
7. Fill in `index.json`
([example](/doc/en/configuring.md#schedules-example),
[documentation](/doc/en/configuring.md#schedules))
8. Run
```console
cargo run --release
```
