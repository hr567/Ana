# [Ana Project](https://gitlab.com/hr567/Ana)

[![pipeline status](https://gitlab.com/hr567/Ana/badges/master/pipeline.svg)](https://gitlab.com/hr567/Ana/commits/master)

A open source judge for ACMers in Rust.

The users of Ana are usually the providers of online judge services rather than ACMers.


## Requirements

* Rust toolchain (Edition 2018 or higher)
* libseccomp

----

Build Dependence:
* Protobuf (protobuf-compiler)
* Clang
* CMake


## Workflow

```
  Judge Task
      |
      ∨
+-----------+    +----------+    +-----------+
|           |    |          |    |           |
|  Builder  | -> |  Runner  | -> |  Checker  |
|           |    |          |    |           |
+-----------+    +----------+    +-----------+
                                       |
                                       ∨
                                  Judge Report
```

The builder, runner ,and checker run on the host directly **without** any isolate, so it is the
user's responsibility to ensure the security of the process. Although the runner is run on the
host, the subprocess will run in an isolated environment.

### Builder

The builder uses a build script to build an executable file from the source file.

Some built-in build script that can be used. The builder will choose one of them from all scripts
according to the language or the suffix of the source file.

Users can use a custom build script instead of the built-in script. The script should not read or
write to any file outside the build directory.

Then the script compiles `SOURCE_FILE` to `EXECUTABLE_FILE`. If the source file is not able to be
compiled to a single static executable file, the script should place the necessary file(s) under
`TARGET_DIR`.

There are some environment variables that can be used in build script:

- `SOURCE_FILE`: The path of the source file.
- `EXECUTABLE_FILE`: The path of the executable file.
- `TARGET_DIR`: The path of the target directory.

### Runner

Ana's runner runs an executable file. It uses a file as the stdin of the program and gets the
output from the stdout of it. Then the runner generates a report about the resource usage of the
program.

The runner can run the executable in an isolated environment and provides a filter for system
calls. It also limits the resource usage of the program.

### Checker

Users can use the built-in comparer for checking the output of the program. The built-in comparer
compares two files/two bytes arrays line by line. By default, the comparer ignores the white space
at the end of the lines and the empty line at the end of the files. This behavior can be overridden
by the problem's configuration.

It is also possible to use a custom script such as `diff` command instead of the comparer. Then the
exit status of the command will be used as the result.

When judging a special judge problem, Ana will use `./spj $INPUT_FILE $OUTPUT_FILE $ANSWER_FILE` by
default to check the output. Users can use a check script for a different check command. Use the
following environment variables for the script:

- `INPUT_FILE`: The file contains input content.
- `OUTPUT_FILE`: The file contains output content.
- `ANSWER_FILE`: The file contains answer content.

[Interactive Problem]: # (TODO: unimplemented)


## Usage

### Run

`cargo run`

Run Ana on the localhost with the default configuration.

**Root permission is needed.**

Run `cargo run -- --help` for more information.

### Test

`cargo test -- --test-threads=1`

Testing Ana needs root permission to read and write to cgroups and implement some other functions.

If you find that the time usage is less than the time limit but the status is TLE, try again with
less `judge_threads`.


### Docker

It is highly recommended to run Ana in a Docker container. Start an Ana server in the background by
following commands: \
`docker run --privileged -p 8800:8800 -d hr567/ana`

For more information and see all supported options: \
`docker run --privileged hr567/ana --help`


## Client

Ana uses gRPC framework to communicate with the client. The protobuf file which has defined the
data structures and services is located in `rpc.proto`.

There is a simple client implementation in `tests/common`.

### Service

Ana's service provides only one function now. The function is `judge`. It receives a task and
returns a stream of reports to its caller.

### Structures

#### Task

The source field includes the source file, including a filename and the content. In most cases, the
source file is a text file. If the build script is set, Ana will use the script to build the
executable file and the language field will be ignored. If the build script field is not set, Ana
will try to choose a suitable built-in build script for the language. The language is provided by
the language field in the task. Otherwise, Ana will infer the language of the source file from the
suffix of the source's filename.

In some special cases, there are multiple files are submitted to the judge system for judging. None
of Ana's build-in build scripts supports judging multiple files now. In these cases, the user
should use an archive as the source file and provide a custom build script. The script contains the
commands of unzipping and compiling the executable.

When the build timeout field is set, the build script will not be able to run longer than the given
timeout. If the build process exceeds the timeout, Ana will kill it and return a CE report.

See the document of `Problem` structure for more information of the problem field.

By default, Ana executes the executable file without any command line arguments after the build
process. But when performing some task whose language do not support compiling to executable file,
the command is different from the executable file. For example, when judging python language. Ana
should use `python $source_file` to execute the program instead of using `./$executable_file`. For
these languages, users can set a custom command and set some arguments. If the user set the args
field but leave the command field empty, the arguments will be added to the executable file. Giving
a custom command without setting arguments means run the command without any parameter. It usually
only produces the same result.

[RunnerConfig]: # (TODO: unimplemented)

#### Problem

Three types of problems are defined in Ana:

- Normal Problem
- Special Judge Problem
- Interactive Problem

Any kind of problems contains a resource limit.

The normal problem is the most common of all problem in OI/ACM contest. It includes multiple cases
of test data. One test case contains input content and answer content. Ana use the input content as
the stdin data of the program and compare the output content with the answer. User is able to set
whether ignore empty lines at the end of the file or white spaces at the end of the lines.

The special judge problem is a problem with a custom checker. The special judge is a program and
will be build to check the output and the answer. Ana will use the build script in the problem for
building the special judge or try to find a suitable built-in script. The build process is the same
as which in building the source code. If the special judge is programed in some language which
needs a launcher like python. Users can specify the check script in order to use a custom check
process. When using the custom check script, use the environment variable for reading the input
file, the output file, and the answer file. If the special judge program exited with code zero or
all the commands in the check script are executed successfully, the program is considered correct.

The interactive problem is similar to the special judge problem. The different between them is
that the interactive problem does not include any test cases. When executable the interactive
checker, Ana will connect the stdin of the program with the stdout of the checker and the stdout
of the program with the stdin of the checker. Users can specify a check script for a different way
to check the output of the program. If the checker exited with code zero or all commands in the
check script are executed successfully, the program is considered correct.


## Workspace

Ana creates a new workspace for each judge task. The structure of the workspace is as shown below:

```
{workspace}
├── config.toml
├── build/
│   ├── build.sh
│   ├── source.c
│   └── target/
├── problem/
│   ├── 0
│   │   ├── answer
│   │   └── input
│   ├── 1
│   │   ├── answer
│   │   └── input
│   └── ..
└── runtime/
```

The config.toml file is generated by Ana.


## TODOs

* Recover from errors
* Add documents
* Use Fuse to reduce memory usage


## License

Ana is published under MIT license, see "[LICENSE](LICENSE)" for more information.
