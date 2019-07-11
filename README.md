# [Ana Project](https://gitlab.com/hr567/Ana)

[![pipeline status](https://gitlab.com/hr567/Ana/badges/master/pipeline.svg)](https://gitlab.com/hr567/Ana/commits/master)

A open source judge for ACMers in Rust.


## Requirements

* Rust toolchain (Edition 2018 or higher)
* Protobuf (Dev files and protobuf-compiler)
* libseccomp


## Usage

### Run

`cargo run`

Run Ana on the localhost with default configuration.

Root permission is needed. If you try to run Ana in
docker container, you must pass `--privileged` to docker.

Run `cargo run -- --help` for more information.

### Test

`cargo test -- --test-threads=1`

Testing Ana needs root permission to
read and write to cgroups and
implement some other functions.

If you find that the time usage is less than time limit
but the status is TLE, try again with less `judge_threads`.


# Client

Ana uses gRPC framework to communicate with client.
The proto file which has defined the data structures
and services is located in `src/rpc/rpc.proto`.

There is a simple client implementation in `tests/common`.


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
* Add documents
* Use Fuse to reduce memory usage


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
