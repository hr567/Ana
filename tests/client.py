import zmq
import time
import docker

with open("example/judge_info.json", 'rt') as f:
    JUDGE_DATA = f.read()

if __name__ == "__main__":
    # docker_cli = docker.from_env()
    # ana = docker_cli.containers.run(
    #     image="hr567/ana:latest",
    #     ports={
    #         "8800/tcp": ('127.0.0.1', 8800)
    #     },
    #     privileged=True,
    #     auto_remove=False,
    #     detach=True,
    # )

    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    socket.connect("tcp://127.0.0.1:8800")

    socket.send(JUDGE_DATA.encode())

    res = socket.recv()
    assert res == b"#0 AC"
    socket.send(b"")
    res = socket.recv()
    assert res == b"#1 AC"
    socket.send(b"")
    res = socket.recv()
    assert res == b"#2 AC"
    socket.send(b"")

    # ana.stop()
