# Running
**ktmuscrap** works on Windows and Linux.

Virtualization through Docker is available.


## Contents
- [Docker on Linux](#docker-on-linux)
- [Windows](#windows)
- [Debian Linux](#debian-linux)


## Docker on Linux
You can run a **ktmuslave** container with
all the dependencies included.

Instructions here.


## Windows
1. Make sure that the [.NET Framework](https://support.microsoft.com/en-us/topic/microsoft-net-framework-4-8-offline-installer-for-windows-9d23f658-3b97-68ab-d013-aa3c3e7495e0)
is installed and is not below 4.6
2. Install [stable Rust](https://www.rust-lang.org/tools/install)
using quick install
3. Download and unpack the repo,
or use **git**:
```bash
git clone https://github.com/kerdl/ktmuscrap
```
4. Go to the code directory or open a **cmd** there:
```bash
cd ktmuscrap-yr2024
```
5. Run and wait for the message
```
cargo run --release
...
0000-00-00 at 00:00:00 [ERROR] before running, see ./data/schedule/index.json and fill in the schedule types manually (src/main.rs:55)
```
6. Open the file `./data/schedule/index.json` and
fill in the field `types`
according to [this documentation](/doc/en/configuring.md#types)
7. Run again
```bash
cargo run --release
```


## Debian Linux
1. Install dependencies
```bash
sudo apt update && sudo apt install git build-essential pkg-config libssl-dev -y
```
2. Install stable Rust using quick install
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. Update environment variables
```bash
source $HOME/.cargo/env
```
4. Clone the repo
```bash
git clone https://github.com/kerdl/ktmuscrap
```
5. Go to the directory
```bash
cd ktmuscrap-yr2024
```
6. Run and wait for the message
```
cargo run --release
...
0000-00-00 at 00:00:00 [ERROR] before running, see ./data/schedule/index.json and fill in the schedule types manually (src/main.rs:55)
```
7. Open the file `./data/schedule/index.json` and
fill in the field `types`
according to [this documentation](/doc/en/configuring.md#types)
```bash
nano data/schedule/index.json
```
8. Run again
```bash
cargo run --release
```
