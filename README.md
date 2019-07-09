# [Ana Project](https://gitlab.com/hr567/Ana)

[![pipeline status](https://gitlab.com/hr567/Ana/badges/master/pipeline.svg)](https://gitlab.com/hr567/Ana/commits/master)

A open source judge for ACMers in Rust.


## Requirements

* Rust toolchain (Edition 2018 or higher)


## Usage

### Run

`cargo run`

Run Ana on the localhost with default configuration.

Needs root permission.

Run `cargo run -- --help` for more information.

### Test

`cargo test`

Testing Ana needs root permission
to read and write to cgroups and other functions.

If you find that the time usage is less than time limit
but the status is TLE, try again with less `judge_threads`.

You can use `cargo test -- --test-threads=1`
on some computers to avoid some testing failures.


## MTP

Ana uses [json](https://www.json.org/) for message transfer.

The unit of time is ns and the unit of memory is byte.

Examples:

* Judge

  Normal problem:

  ```json
  {
    "source": {
      "language": "cpp.gxx",
      "code": "..."
    },
    "problem": {
      "type": "Normal",
      "time_limit": 1000000000,
      "memory_limit": 33554432,
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
    "source": {
      "language": "cpp.gxx",
      "code": "..."
    },
    "problem": {
      "type": "Special",
      "time_limit": 1000000000,
      "memory_limit": 33554432,
      "spj": {
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
    "index": 0,
    "result": "AC",
    "time_usage": 800000000,
    "memory_usage": 1258291
  }
  ```


## Workspace
```
workspace
├── problem
│   ├── 0
│   │   ├── answer    // file contains answer
│   │   ├── input     // file contains input
│   │   └── output    // file contains input
│   ├── 1
│   │   └── ..        // same as 0
│   ├── 2
│   │   └── ..        // same as 0
│   ├── ..            // more test cases
│   └── spj           // same as source (if spj is available)
│       ├── spj
│       ├── lang
│       └── source
├── runtime           // chroot directory (empty)
|   └── main          // executable file
└── source            // source code
```


## TODOs

* Recover from errors
* Limit max concurrent tasks
* Add documents
* Use Fuse to reduce memory usage


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
