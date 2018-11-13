# [Ana Project](https://gitlab.com/hr567/Ana)

A judge for ACM.


## Status

Ana is ALMOST done.
But there are some known bugs/todos:

* Ana in Docker can not work now (May be caused by /tmp dir)
* Need more test
* Need documents


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
