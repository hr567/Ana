# [Ana Project](https://gitlab.com/hr567/Ana)

[![pipeline status](https://gitlab.com/hr567/Ana/badges/master/pipeline.svg)](https://gitlab.com/hr567/Ana/commits/master)

A open source judge for ACMers in Rust


## Requirements

* Rust toolchain (Edition 2018 or higher)
* ZeroMQ (such as libzmq-dev on Ubuntu)


## Usage

`cargo test`

Please note that testing Ana needs root permission
to read and write to cgroups.

If you find that the time usage is less than time limit
but the status is TLE, try again with less `max_threads`.

You can use `cargo test -- --test-threads=1`
on some computers to avoid some testing mistake.

`cargo run` to run Ana on the localhost.
Needs root permission too.

Run `cargo run -- --help` for more information.


## MTP

Ana uses ZeroMQ to communicate with online judge server.

It uses a PULL and a PUSH to receive and send message.
A message is a json string and defined as following examples:

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
      "time_limit": 1000000000,
      "memory_limit": 33554432,
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
      "time_limit": 1000000000,
      "memory_limit": 33554432,
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
    "time": 800000000,
    "memory": 1258291
  }
  ```


## Workspace
```
workspace
├── id                // Judge ID
├── problem
│   ├── time_limit    // time limit in ns
│   ├── memory_limit  // memory limit in byte
│   ├── 0
│   │   ├── answer    // file contains answer
│   │   ├── input     // file contains input
│   │   └── output    // file contains input
│   ├── 1
│   │   └── ..        // same as 0
│   ├── 2
│   │   └── ..        // same as 0
│   ├── ..            // more test cases
│   └── spj           // same as source if spj is available
│       ├── spj
│       ├── lang
│       └── source
├── runtime           // chroot directory (empty)
|   └── main          // executable file
└── source            // directory for source code and compiled program
    ├── lang          // language of the code
    └── source        // source code
```


## TODOs

* Recover from errors
* Limit max concurrent tasks
* Judge test cases concurrently
* Add documents
* Cache problems
* Use Fuse to reduce memory usage


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
