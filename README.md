# [Ana Project](https://gitlab.com/hr567/Ana)

A open source judge for ACMers in Rust


## Status

Ana is ALMOST done.
We need more test and documents
and a few more functions.


## Requirements

* Docker

To build Ana at local:

* Rust toolchain (Edition 2018 or higher)
* ZeroMQ (such as libzmq-dev)
* lrun (https://github.com/quark-zju/lrun)


## Usage

### Docker

`docker build -t hr567/ana .`

`docker run --privileged --port 8800:8800 hr567/ana`

After starting the container,
you can test it using `python3 tests/basic_test.py`
to test if it work correctly.

### Normal

Use `cargo test` to run unit test.

`cargo run` to run ana on the localhost.


## MTP

Ana use ZeroMQ to communicate with online judge server.

It use a REP to receive and send message.
A message is a json string and
is defined as following examples:

* Judge

  Normal problem:

  Set type to "normal", and keep the checker empty.

  ```json
  {
    "id": "b6555832ef2111e8bc847470fd3b4381",
    "source": {
      "language": "cpp.gxx",
      "code": "..."
    },
    "problem": {
      "problem_type": "normal",
      "time_limit": 1.0,
      "memory_limit": 32.0,
      "checker": {
        "language": "",
        "code": ""
      },
      "test_cases": [
        {
          "input": "1 1",
          "answer": "2"
        },
        {
          "input": "13 5\n14 7\n23 45",
          "answer": "18\n21\n68"
        },
        {
          "input": "24 3\n17 -5\n123 945",
          "answer": "27\n12\n1068"
        }
      ]
    }
  }
  ```

  Problem with special judge:

  ```json
  {
    "id": "b6555832ef2111e8bc847470fd3b4381",
    "source": {
      "language": "cpp.gxx",
      "code": "..."
    },
    "problem": {
      "problem_type": "spj",
      "time_limit": 1.0,
      "memory_limit": 32.0,
      "checker": {
        "language": "cpp.gxx",
        "code": "..."
      },
      "test_cases": [
        {
          "input": "1 1",
          "answer": "2"
        },
        {
          "input": "13 5\n14 7\n23 45",
          "answer": "18\n21\n68"
        },
        {
          "input": "24 3\n17 -5\n123 945",
          "answer": "27\n12\n1068"
        }
      ]
    }
  }
  ```
* Report

  ```json
  {
    "id": "b6555832ef2111e8bc847470fd3b4381",
    "case_index": 0,
    "status": "AC",
    "time": 0.8,
    "memory": 1024
  }
  ```


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
