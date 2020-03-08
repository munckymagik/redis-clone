# Rust::clone(Redis)

⚠️ **NOT FOR PRODUCTION USE - toy only** ⚠️

This is a clone of a portion of Redis for two reasons:

* Give me a structured way of getting to know the real Redis source and learning what makes it so good.
* Provide a decent sized project to exercise Rust's new async/await features and async I/O.

## Dependencies

* Any version of Rust with "Rust 2018 edition" compatibility
  * Get Rust from https://rustup.rs/
* Redis itself
  * The API/functional tests can be run against _real_ Redis to cross validate
  * We can use `redis-cli` to connect to and interact with the clone
  * `redis-benchmark` can be used to compare performance of the clone with the real one
* Ruby 2.* - for the functional test suite

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

Use the `redis-benchmark` tool, but make sure to only specify tests for commands the clone supports:

```
redis-benchmark -p 8080 -t SET,GET,INCR
```
