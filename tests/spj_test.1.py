#!/usr/bin/env python3
"""Test for Ana Project

Please make sure you have built docker image correctly and tag it as `hr567/ana:latest`.
The tests will use the images to test if judging function work right.
If you haven't built the image. See README for more information.
"""

import json
import unittest
import uuid

import docker
import zmq

PROBLEM_TEST_CASE_COUNT = 3
with open("example/spj_problem.json", 'rt') as f:
    PROBLEM = json.loads(f.read())
    with open("example/spj.1.cpp") as f:
        PROBLEM["checker"]["code"] = f.read()


def generate_judge(source):
    judge_info = {
        "id": uuid.uuid4().hex,
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
        self.sender = self.context.socket(zmq.PUSH)
        self.receiver = self.context.socket(zmq.PULL)

        self.container.reload()
        ip = self.container.attrs["NetworkSettings"]["IPAddress"]

        self.sender.connect(f"tcp://{ip}:8800")
        self.receiver.connect(f"tcp://{ip}:8801")

    def tearDown(self):
        self.container.remove()
        self.sender.close()
        self.receiver.close()
        self.context.destroy()
        self.cli.close()

    def test_ac(self):
        with open("example/source.cpp", 'rt') as f:
            source = {
                "language": "cpp.gxx",
                "code": f.read()
            }
        judge_info = generate_judge(source)
        self.sender.send(judge_info.encode())
        for i in range(PROBLEM_TEST_CASE_COUNT):
            report = self.receiver.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["id"], json.loads(judge_info)["id"])
            self.assertEqual(report["case_index"], i)
            self.assertEqual(report["status"], "AC")
            self.assertLessEqual(report["time"], 1.0)
            self.assertLessEqual(report["memory"], 32.0)
        report = self.receiver.recv().decode()
        report = json.loads(report)
        self.assertEqual(report["id"], json.loads(judge_info)["id"])
        self.assertEqual(report["case_index"], PROBLEM_TEST_CASE_COUNT)
        self.assertEqual(report["status"], "AC")
        self.assertLessEqual(report["time"], 1.0)
        self.assertLessEqual(report["memory"], 32.0)
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_mle(self):
        with open("example/source.mle.cpp", 'rt') as f:
            source = {
                "language": "cpp.gxx",
                "code": f.read()
            }
        judge_info = generate_judge(source)
        self.sender.send(judge_info.encode())
        for i in range(PROBLEM_TEST_CASE_COUNT):
            report = self.receiver.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["id"], json.loads(judge_info)["id"])
            self.assertEqual(report["case_index"], i)
            self.assertEqual(report["status"], "MLE")
            self.assertLessEqual(report["time"], 1.0)
            self.assertAlmostEqual(
                report["memory"], 32.0, delta=0.01)
        report = self.receiver.recv().decode()
        report = json.loads(report)
        self.assertEqual(report["id"], json.loads(judge_info)["id"])
        self.assertEqual(report["case_index"], PROBLEM_TEST_CASE_COUNT)
        self.assertEqual(report["status"], "MLE")
        self.assertLessEqual(report["time"], 1.0)
        self.assertAlmostEqual(
            report["memory"], 32.0, delta=0.01)
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_re(self):
        with open("example/source.re.cpp", 'rt') as f:
            source = {
                "language": "cpp.gxx",
                "code": f.read()
            }
        judge_info = generate_judge(source)
        self.sender.send(judge_info.encode())
        for i in range(PROBLEM_TEST_CASE_COUNT):
            report = self.receiver.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["id"], json.loads(judge_info)["id"])
            self.assertEqual(report["case_index"], i)
            self.assertEqual(report["status"], "RE")
            self.assertLessEqual(report["time"], 1.0)
            self.assertLessEqual(report["memory"], 32.0)
        report = self.receiver.recv().decode()
        report = json.loads(report)
        self.assertEqual(report["id"], json.loads(judge_info)["id"])
        self.assertEqual(report["case_index"], PROBLEM_TEST_CASE_COUNT)
        self.assertEqual(report["status"], "RE")
        self.assertLessEqual(report["time"], 1.0)
        self.assertLessEqual(report["memory"], 32.0)
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_tle(self):
        with open("example/source.tle.cpp", 'rt') as f:
            source = {
                "language": "cpp.gxx",
                "code": f.read()
            }
        judge_info = generate_judge(source)
        self.sender.send(judge_info.encode())
        for i in range(PROBLEM_TEST_CASE_COUNT):
            report = self.receiver.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["id"], json.loads(judge_info)["id"])
            self.assertEqual(report["case_index"], i)
            self.assertEqual(report["status"], "TLE")
            self.assertAlmostEqual(report["time"], 1.0, delta=0.05)
            self.assertLessEqual(report["memory"], 32.0)
        report = self.receiver.recv().decode()
        report = json.loads(report)
        self.assertEqual(report["id"], json.loads(judge_info)["id"])
        self.assertEqual(report["case_index"], PROBLEM_TEST_CASE_COUNT)
        self.assertEqual(report["status"], "TLE")
        self.assertAlmostEqual(report["time"], 1.0, delta=0.05)
        self.assertLessEqual(report["memory"], 32.0)
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)

    def test_wa(self):
        with open("example/source.wa.cpp", 'rt') as f:
            source = {
                "language": "cpp.gxx",
                "code": f.read()
            }
        judge_info = generate_judge(source)
        self.sender.send(judge_info.encode())
        for i in range(PROBLEM_TEST_CASE_COUNT):
            report = self.receiver.recv().decode()
            report = json.loads(report)
            self.assertEqual(report["id"], json.loads(judge_info)["id"])
            self.assertEqual(report["case_index"], i)
            self.assertEqual(report["status"], "WA")
            self.assertLessEqual(report["time"], 1.0)
            self.assertLessEqual(report["memory"], 32.0)
        report = self.receiver.recv().decode()
        report = json.loads(report)
        self.assertEqual(report["id"], json.loads(judge_info)["id"])
        self.assertEqual(report["case_index"], PROBLEM_TEST_CASE_COUNT)
        self.assertEqual(report["status"], "WA")
        self.assertLessEqual(report["time"], 1.0)
        self.assertLessEqual(report["memory"], 32.0)
        self.assertEqual(self.container.wait(timeout=5)["StatusCode"], 0)


if __name__ == "__main__":
    unittest.main()
