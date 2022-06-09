### About

Simple TCP server implementation with [Uppercut](https://github.com/sergey-melnychuk/uppercut).

### Benchmarks

#### Hardware

##### Local

Intel Core i7 2.6 GHz (4-core) / 16 GB RAM

##### Cloud

Client: 2-core / 8 GB RAM

Server: 8-core / 32 GM RAM

#### Setup

Server:

```
sudo apt update && sudo apt install -y build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

git clone https://github.com/sergey-melnychuk/uppercut-lab.git
cd uppercut-lab/uppercut-mio-server
cargo build --release

./target/release/uppercut-mio-server
```

Client:

```shell
sudo apt install build-essential libssl-dev git unzip -y
git clone https://github.com/wg/wrk.git
cd wrk
make
sudo cp wrk /usr/local/bin

wrk -d 1m -c 128 -t 4 http://$HOST:9000/
```

```
sudo apt install -y linux-tools-common linux-tools-generic linux-tools-`uname -r`
cargo install flamegraph
```

```
# Cargo.toml
[profile.release]
debug = true
```

#### Results

Best of 3 runs

##### uppercut-mio-server

```
Running 1m test @ http://64.225.96.110:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.65ms    2.49ms  28.93ms   83.39%
    Req/Sec    32.82k     7.27k   74.61k    57.73%
  7841381 requests in 1.00m, 0.91GB read
Requests/sec: 130527.84
Transfer/sec: 15.44MB
```

##### actix-web

```
Running 1m test @ http://64.225.96.110:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.66ms    2.49ms  25.51ms   83.42%
    Req/Sec    31.49k     8.25k   53.36k    54.50%
  7522556 requests in 1.00m, 0.85GB read
Requests/sec: 125200.09
Transfer/sec: 14.57MB
```

##### tide

```
Running 1m test @ http://64.225.96.110:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    11.12ms   14.32ms  56.24ms   78.34%
    Req/Sec    12.34k     1.52k   16.47k    65.75%
  2947430 requests in 1.00m, 340.13MB read
Requests/sec: 49109.17
Transfer/sec: 5.67MB
```
