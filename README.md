# [Ana Project](https://gitlab.com/hr567/Ana)

A open source judge for ACMers in Rust


## Status

Ana is ALMOST done.
We need more test and documents
and a few more functions.


## Requirements

* Rust toolchain (Edition 2018 or higher)
* ZeroMQ (such as libzmq-dev)
* lrun (https://github.com/quark-zju/lrun)


## Usage

### Docker

`docker build -t hr567/ana .`

`docker run --privileged --port 8800:8800 hr567/ana`

After starting the container,
you can test it using `python3 tests/basic_test.py` to test if it work correctly.

### Normal

Use `cargo test` to run unit test.

`cargo run` to run ana on the localhost.


## MTP

Ana use ZeroMQ to communicate with online judge server.

It use a REP to receive and send message.
A message is a json string and
is defined as following examples:

* Judge

  ```json
  {
    "language": "cpp.gxx",
    "source": "...",
    "problem": {
      time_limit: 1.0,
      memory_limit: 64.0, // Mb
      test_cases: [
        ["1 1", "2"],
        ["2 3", "5"],
        ["1 1\n2 3", "2\n5"]
      ]
    }
  }
  ```
* Report

  ```json
  {
    "status": "AC",
    "time": 0.8,
    "memory": 1024 // bytes
  }
  ```


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
