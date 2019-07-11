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

Run Ana on the localhost with the default configuration.

**Root permission is needed.**

Run `cargo run -- --help` for more information.

### Test

`cargo test -- --test-threads=1`

Testing Ana needs root permission to
read and write to cgroups and
implement some other functions.

If you find that the time usage is less than
the time limit but the status is TLE,
try again with less `judge_threads`.


### Docker

It is highly recommended to run Ana in a Docker container.
Start an Ana server in the background by following commands: \
`docker run --privileged -d hr567/ana run`

And you can also use other commands, use \
`docker run --privileged hr567/ana --help` \
or \
`docker run --privileged hr567/ana -- --help`

for more information and see
all supported commands and configurations.


## Client

Ana uses gRPC framework to communicate with the client.
The protobuf file which has defined the data structures
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

Ana is published under MIT license,
see "[LICENSE](LICENSE)" for more information.
