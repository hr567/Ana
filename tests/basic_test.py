#!/usr/bin/env python3
"""Test for Ana Project

Please make sure you have built docker image correctly and tag it as `hr567/ana:latest`.
The tests will use the images to test if judging function work right.
If you haven't built the image. See README for more information.
"""

import json
import unittest

import docker
import zmq

PROBLEM_TEST_CASE_COUNT = 3
with open("example/problem.json", 'rt') as f:
    PROBLEM = json.loads(f.read())


def generate_judge(source):
    judge_info = {
        "language": "cpp.gxx",
        "source": source,
        "problem": PROBLEM,
    }
    return json.dumps(judge_info)


class AnaTest(unittest.TestCase):

    def setUp(self):
        self.cli = docker.from_env()
        self.container = self.cli.containers.run(
            image="hr567/ana",
            privileged=True,
            detach=True,
        )

        self.context = zmq.Context()
        self.socket = self.context.socket(zmq.REQ)
        self.container.reload()
        ip = self.container.attrs["NetworkSettings"]["IPAddress"]
        port = 8800
        self.socket.connect(f"tcp://{ip}:{port}")

    def tearDown(self):
        self.container.remove(force=True)
        self.socket.close()
        self.context.destroy()
        self.cli.close()

    def test_ce(self):
        with open("example/source.ce.cpp", 'rt') as f:
            source = f.read()
        judge_info = generate_judge(source)
        self.socket.send(judge_info.encode())
        for _ in range(PROBLEM_TEST_CASE_COUNT):
            report = self.socket.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["status"], "CE")
            self.assertEqual(report["time"], 0.0)
            self.assertEqual(report["memory"], 0)
            self.socket.send(b"")
            break
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_ac(self):
        with open("example/source.cpp", 'rt') as f:
            source = f.read()
        judge_info = generate_judge(source)
        self.socket.send(judge_info.encode())
        for _ in range(PROBLEM_TEST_CASE_COUNT):
            report = self.socket.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["status"], "AC")
            self.assertLessEqual(report["time"], 1.0)
            self.assertLessEqual(report["memory"], 32 * 1024 * 1024)
            self.socket.send(b"")
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_mle(self):
        with open("example/source.mle.cpp", 'rt') as f:
            source = f.read()
        judge_info = generate_judge(source)
        self.socket.send(judge_info.encode())
        for _ in range(PROBLEM_TEST_CASE_COUNT):
            report = self.socket.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["status"], "MLE")
            self.assertLessEqual(report["time"], 1.0)
            self.assertAlmostEqual(
                report["memory"], 32 * 1024 * 1024, delta=1024)
            self.socket.send(b"")
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_re(self):
        with open("example/source.re.cpp", 'rt') as f:
            source = f.read()
        judge_info = generate_judge(source)
        self.socket.send(judge_info.encode())
        for _ in range(PROBLEM_TEST_CASE_COUNT):
            report = self.socket.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["status"], "RE")
            self.assertLessEqual(report["time"], 1.0)
            self.assertLessEqual(report["memory"], 32 * 1024 * 1024)
            self.socket.send(b"")
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_tle(self):
        with open("example/source.tle.cpp", 'rt') as f:
            source = f.read()
        judge_info = generate_judge(source)
        self.socket.send(judge_info.encode())
        for _ in range(PROBLEM_TEST_CASE_COUNT):
            report = self.socket.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["status"], "TLE")
            self.assertAlmostEqual(report["time"], 1.0, delta=0.05)
            self.assertLessEqual(report["memory"], 32 * 1024 * 1024)
            self.socket.send(b"")
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_wa(self):
        with open("example/source.wa.cpp", 'rt') as f:
            source = f.read()
        judge_info = generate_judge(source)
        self.socket.send(judge_info.encode())
        for _ in range(PROBLEM_TEST_CASE_COUNT):
            report = self.socket.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["status"], "WA")
            self.assertLessEqual(report["time"], 1.0)
            self.assertLessEqual(report["memory"], 32 * 1024 * 1024)
            self.socket.send(b"")
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)


if __name__ == "__main__":
    unittest.main()
