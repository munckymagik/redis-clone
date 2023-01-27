# Rust::clone(Redis)

![Rust](https://github.com/munckymagik/redis-clone/workflows/Rust/badge.svg)

⚠️ **NOT FOR PRODUCTION USE - toy only** ⚠️

This is a clone of Redis.

## Features

It supports a tiny subset of the features in _real_ Redis.

The current goal is to support [just enough to run the opensource features of Sidekiq](https://github.com/munckymagik/redis-clone/issues/1).

I have no serious aims to support the full features of _real_ Redis.

## Why?

It is for-fun only and not intended to be a "[rewrite it in Rust](https://github.com/ansuz/RIIR)" project 😀.

It exists because I wanted:

* a structured way of getting to know the [_real_ Redis source](https://github.com/antirez/redis).
* to gain a deeper understanding of how Redis works and why it performs as it does.
* a sufficiently non-trivial project to exercise Rust's async/await and async I/O facilities.

However, my intention is to write fully production quality code, as far as possible. So it might be
useful to seed new services that could be driven using Redis-style commands using the [RESP protocol](https://redis.io/topics/protocol).

## Dependencies

* Any version of Rust with "Rust 2018 edition" compatibility
  * You can get Rust using https://rustup.rs/
* Redis itself
  * The API/functional tests can be run against _real_ Redis to cross validate
  * We can use `redis-cli` to connect to and interact with the clone
  * `redis-benchmark` can be used to compare performance of the clone with the real one
* Ruby 3.* - for the functional test suite

## Running

Clone the project then:

```shell
cargo run --release
```

You should see the familiar Redis ASCII art and a comforting message that the clone is listening on port 8080.

```
[2020-01-17T16:16:44Z WARN  redis_clone] oO0OoO0OoO0Oo Redis Clone is starting oO0OoO0OoO0Oo
[2020-01-17T16:16:44Z WARN  redis_clone] Redis version=0.1.0, bits=64, pid=74915, just started
                _._
           _.-``__ ''-._
      _.-``    `.  `_.  ''-._           Redis Clone 0.1.0 64 bit
  .-`` .-```.  ```\\/    _.,_ ''-._
 (    '      ,       .-`  | `,    )     Running in standalone mode
 |`-._`-...-` __...-.``-._|'` _.-'|     Port: 8080
 |    `-._   `._    /     _.-'    |     PID: 74915
  `-._    `-._  `-./  _.-'    _.-'
 |`-._`-._    `-.__.-'    _.-'_.-'|
 |    `-._`-._        _.-'_.-'    |     https://github.com/munckymagik/redis-clone
  `-._    `-._`-.__.-'_.-'    _.-'
 |`-._`-._    `-.__.-'    _.-'_.-'|
 |    `-._`-._        _.-'_.-'    |
  `-._    `-._`-.__.-'_.-'    _.-'
      `-._    `-.__.-'    _.-'
          `-._        _.-'
              `-.__.-'

[2020-01-17T16:16:44Z INFO  redis_clone::server] Listening at ("127.0.0.1", 8080)
```

## Using

You can use the `redis-cli` command to connect to the clone:

```shell
redis-cli -p 8080
```

Then you can see what commands are available:

```
127.0.0.1:8080> COMMAND
 1) 1) "get"
    2) (integer) 2
 2) 1) "set"
    2) (integer) -3
# ... snip ...
 23) 1) "keys"
    2) (integer) 2
```

## Testing

There are two kinds of tests:

* Unit tests within the Rust source code
* A suite of functional tests that use Ruby and the Ruby Redis Gem to test the supported commands

To run the Rust tests for all packages in the workspace:

```shell
cargo test
```

The for the functional tests:

```shell
cd tests/functional
bundle install
./bin/rspec
```

To run the test suite against _real_ Redis do this:

```
TEST_REAL_REDIS=1 bundle exec rspec
```

## Benchmarking

The `redis-benchmark` tool can be used, but make sure to
  1. only specify tests for commands the clone supports
  2. specify port 8080

```
redis-benchmark -p 8080 -t SET,GET,INCR
```

A script that runs the supported tests against both _real_ Redis the clone is available in:

```
./benches/clone-vs-real.sh
```

## Development

Use the scripts under the `./scripts` folder to watch, build and re-run the tests.

They depend on [cargo-watch](https://github.com/passcod/cargo-watch) so make sure to install it
first with `cargo install cargo-watch`. Then

1. In one terminal watch and rebuild the server:
   ```bash
   ./scripts/watch-server
   ```
2. In another terminal watch the server and re-run the tests. This also runs the tests against
   "real" Redis so make sure that's running too:
   ```bash
   ./scripts/watch-feature-tests
   ```
