import zmq

with open("example/judge_info.json", 'rt') as f:
    JUDGE_DATA = f.read()

if __name__ == "__main__":
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
