### About

Simple TCP server implementation with [Uppercut](https://github.com/sergey-melnychuk/uppercut).

### Benchmarks

#### Hardware

```
Dual-Core Intel Core i5 1,4 GHz
1 Processor x 2 Cores x (Hyper-Threading)
Memory:	4 GB
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
cd baseline
cargo build --release
```

Run: `target/release/uppercut-mio-server` or `baseline/target/release/hello-world`.

#### Command

`wrk -d 10s -c 128 -t 4 http://127.0.0.1:9000/`

#### Results

Single-threaded server (baseline):
```
Running 10s test @ http://127.0.0.1:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.30ms    1.74ms  32.19ms   96.32%
    Req/Sec    14.86k     2.90k   18.95k    78.25%
  591507 requests in 10.01s, 49.64MB read
Requests/sec:  59069.18
Transfer/sec:      4.96MB
```

Actor-based multi-threaded server:
```
$ wrk -d 10s -c 128 -t 4 http://127.0.0.1:9000/
Running 10s test @ http://127.0.0.1:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   132.07us  287.89us  14.06ms   95.81%
    Req/Sec    28.79k     4.78k   38.82k    72.00%
  573622 requests in 10.07s, 50.88MB read
Requests/sec:  56943.37
Transfer/sec:      5.05MB
```

#### Benchmarking on VMs

Server: 8 vCPUs / 16GB RAM / Ubuntu 18.04.3 (LTS) x64

Client: 4 vCPUs / 8GB RAM / Ubuntu 18.04.3 (LTS) x64

1st round:

```
# uppercut-mio-server
root@ubuntu-s-4vcpu-8gb-fra1-01:~/wrk# wrk -d 1m -c 128 -t 4 http://server:9000/
Running 1m test @ http://server:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.89ms  762.53us  21.86ms   97.21%
    Req/Sec    36.42k     7.22k   48.45k    71.79%
  8698418 requests in 1.00m, 771.48MB read
Requests/sec: 144912.44
Transfer/sec:     12.85MB
```

```
# baseline (actix-web helloworld)
root@ubuntu-s-4vcpu-8gb-fra1-01:~/wrk# wrk -d 1m -c 128 -t 4 http://server:9000/
Running 1m test @ http://server:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.85ms  342.86us  23.11ms   90.51%
    Req/Sec    35.81k     4.43k   47.26k    73.67%
  8553644 requests in 1.00m, 0.97GB read
Requests/sec: 142531.52
Transfer/sec:     16.58MB
```

2nd round:

```
# uppercut-mio-server
root@ubuntu-s-4vcpu-8gb-fra1-01:~/wrk# wrk -d 1m -c 128 -t 4 http://server:9000/
Running 1m test @ http://server:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.94ms  556.65us  20.68ms   95.74%
    Req/Sec    33.31k     4.29k   44.44k    68.25%
  7955514 requests in 1.00m, 705.59MB read
Requests/sec: 132565.78
Transfer/sec:     11.76MB
```

```
# baseline (actix-web helloworld)
root@ubuntu-s-4vcpu-8gb-fra1-01:~/wrk# wrk -d 1m -c 128 -t 4 http://server:9000/
Running 1m test @ http://server:9000/
  4 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.05ms  454.35us  24.64ms   87.89%
    Req/Sec    29.59k     2.80k   39.66k    68.92%
  7067435 requests in 1.00m, 822.28MB read
Requests/sec: 117761.15
Transfer/sec:     13.70MB
```

Extra round (with `-t 8` instead of `-t 4`):

```
# uppercut-mio-server
root@ubuntu-s-4vcpu-8gb-fra1-01:~/wrk# wrk -d 1m -c 128 -t 8 http://server:9000/
Running 1m test @ http://server:9000/
  8 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.04ms    2.47ms  25.21ms   90.84%
    Req/Sec    10.71k     1.69k   15.47k    67.40%
  5119265 requests in 1.00m, 454.04MB read
Requests/sec:  85282.78
Transfer/sec:      7.56MB
```

```
# baseline (actix-web helloworld)
root@ubuntu-s-4vcpu-8gb-fra1-01:~/wrk# wrk -d 1m -c 128 -t 8 http://server:9000/
Running 1m test @ http://server:9000/
  8 threads and 128 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.91ms  363.12us  17.83ms   89.32%
    Req/Sec    17.16k     1.92k   22.95k    72.85%
  8198160 requests in 1.00m, 0.93GB read
Requests/sec: 136595.84
Transfer/sec:     15.89MB
```
