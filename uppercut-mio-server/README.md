### About

Simple TCP server implementation with [Uppercut](https://github.com/sergey-melnychuk/uppercut).

### Benchmarks

#### Hardware

```
Intel Core i7 2.6 GHz (4-core) / 16 GB RAM
```

#### Setup

Cargo ([rustup](https://rustup.rs/)):

```
sudo apt update && sudo apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

WRK ([link](https://github.com/wg/wrk/wiki/Installing-wrk-on-Linux)):

```shell
sudo apt-get install build-essential libssl-dev git unzip -y
git clone https://github.com/wg/wrk.git wrk
cd wrk
make
sudo cp wrk /usr/local/bin
```

Build:

```shell
git clone https://github.com/sergey-melnychuk/uppercut-lab
cd uppercut-lab/uppercut-mio-server
caro build --release
```

#### Command

`wrk -d 10s -c 128 -t 4 http://127.0.0.1:9000/`

#### Results

Best of 3 runs for backend:

##### actix-web

```
Running 1m test @ http://127.0.0.1:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.93ms  149.64us  16.64ms   92.30%
    Req/Sec    34.50k     3.38k   41.80k    49.46%
  8239354 requests in 1.00m, 0.94GB read
Requests/sec: 137313.75
Transfer/sec: 15.98MB
```

##### uppercut-mio-server

```
Running 1m test @ http://127.0.0.1:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.86ms  387.64us  17.62ms   89.88%
    Req/Sec    32.77k     3.07k   39.14k    72.92%
  7825935 requests in 1.00m, 0.90GB read
Requests/sec: 130383.46
Transfer/sec: 15.42MB
```

##### tide

```
Running 1m test @ http://127.0.0.1:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    11.33ms   14.58ms  59.90ms   79.29%
    Req/Sec    12.49k     1.55k   18.67k    70.38%
  2983737 requests in 1.00m, 344.32MB read
Requests/sec: 49691.66
Transfer/sec: 5.73MB
```

