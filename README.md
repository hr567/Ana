# [Ana Project](https://gitlab.com/hr567/Ana)

[![pipeline status](https://gitlab.com/hr567/Ana/badges/master/pipeline.svg)](https://gitlab.com/hr567/Ana/commits/master)

A open source judge for ACMers in Rust


## Status

Ana is ALMOST done.
We need more test and documents
and a few more functions.


## Requirements

* Rust toolchain (Edition 2018 or higher)
* ZeroMQ (such as libzmq-dev on Ubuntu)


## Usage

`cargo test`

Please note that testing Ana needs root permission
to read and write to cgroups.

You can use `cargo test -- --test-threads=1`
on some computers to avoid some testing mistake.

`cargo run` to run Ana on the localhost.
Needs root permission too.

Run `cargo run -- --help` for more information.


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
    "memory": 1.2
  }
  ```


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
