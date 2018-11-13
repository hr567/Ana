# [Ana Project](https://gitlab.com/hr567/Ana)

A judge for ACM.


## Status

Ana is still heavily under development.
It may take a long time to be completed.


## Requirements

* Rust toolchain (Edition 2018 or higher)
* ZeroMQ (such as libzmq-dev)
* lrun (https://github.com/quark-zju/lrun)


## Building

`docker build -t hr567/ana .`


## Testing

`docker run -d hr567/ana`

`python tests/client.py`


## License

Ana is published under MIT licence,
see "[LICENSE](LICENSE)" for more information.
