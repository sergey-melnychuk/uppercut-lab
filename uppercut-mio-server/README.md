### About

Simple TCP server implementation with [Uppercut](https://github.com/sergey-melnychuk/uppercut).

### Benchmarks

#### Hardware

```
Dual-Core Intel Core i5 1,4 GHz
1 Processor x 2 Cores x (Hyper-Threading)
Memory:	4 GB
```

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
