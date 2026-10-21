from pathlib import Path
import json
import sys
import tempfile
import unittest
from experiments.evaluator.spec import Command, Suite
from experiments.evaluator.trial import evaluate
from experiments.evaluator.workspace import git, initialize_git

RESPONSE = 'print(json.dumps({"schema_version":1,"claimed_complete":True,"summary":"done"}))'


class EvaluatorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        (self.repo / "value.txt").write_text("wrong")
        (self.repo / "policy.txt").write_text("fixed")
        initialize_git(self.repo)
        self.commit = git(self.repo, "rev-parse", "HEAD").decode().strip()
        self.grader = self.root / "grader.py"
        self.grader.write_text('from pathlib import Path\nimport json\n'
                               'ok = Path("value.txt").read_text() == "correct"\n'
                               'print(json.dumps({"accepted":ok,"reason":"correct value required"}))\n'
                               'raise SystemExit(0 if ok else 1)\n')
        self.config = {"schema_version": 1, "agent_identity": "unit-test-agent",
                       "agent": {"argv": [sys.executable, "-c", ""], "timeout_seconds": 5},
                       "tasks": [{"id": "repair", "repo": str(self.repo), "commit": self.commit,
                                  "prompt": "Set value.txt to correct", "editable_files": ["value.txt"],
                                  "regression": [{"argv": [sys.executable, "-c", "pass"], "timeout_seconds": 5}],
                                  "grader": str(self.grader), "grader_timeout_seconds": 5}]}

    def tearDown(self):
        self.temp.cleanup()

    def suite(self, body="", response=RESPONSE):
        self.config["agent"]["argv"][-1] = 'import json,sys\nfrom pathlib import Path\nr=json.load(sys.stdin)\n' + body + '\n' + response
        path = self.root / "suite.json"
        path.write_text(json.dumps(self.config))
        return Suite.load(path)

    def trial(self, body="", response=RESPONSE):
        suite = self.suite(body, response)
        return evaluate(suite, suite.tasks[0], "Investigate before editing.", self.root / "trials")

    def test_noop_completion_claim_does_not_pass(self):
        report = self.trial()
        self.assertEqual(report["status"], "rejected")
        self.assertTrue(report["agent"]["claimed_complete"])
        self.assertEqual(report["score"], 0.0)
        self.assertIsNone(report["agent"]["usage"]["cost_usd"])

    def test_correct_edit_passes_without_mutating_source(self):
        report = self.trial('Path("value.txt").write_text("correct")')
        self.assertEqual(report["status"], "accepted", report)
        self.assertEqual((self.repo / "value.txt").read_text(), "wrong")
        self.assertEqual(git(self.repo, "rev-parse", "HEAD").decode().strip(), self.commit)
        self.assertTrue(Path(report["report_path"]).is_file())

    def test_forbidden_edit_is_rejected_even_with_correct_answer(self):
        report = self.trial('Path("value.txt").write_text("correct")\nPath("policy.txt").write_text("weakened")')
        self.assertEqual(report["status"], "policy_violation")
        self.assertEqual(report["forbidden_files"], ["policy.txt"])

    def test_grading_ignores_agent_git_index_and_build_output(self):
        report = self.trial('import subprocess\nPath("value.txt").write_text("correct")\n'
                            'subprocess.run(["git","add","value.txt"],check=True)\n'
                            'Path("target").mkdir()\nPath("target/fake-grade").write_text("pass")')
        self.assertEqual(report["status"], "accepted", report)

    def test_symlink_patch_is_rejected(self):
        report = self.trial('Path("value.txt").unlink()\nPath("value.txt").symlink_to("policy.txt")')
        self.assertEqual(report["status"], "policy_violation")

    def test_timeout_and_bad_protocol_are_distinct(self):
        self.config["agent"]["timeout_seconds"] = 1
        self.assertEqual(self.trial('import time\ntime.sleep(10)')["status"], "timeout")
        self.assertEqual(self.trial(response='print("not json")')["status"], "protocol_error")

    def test_invalid_usage_is_not_reported_as_zero(self):
        response = 'print(json.dumps({"schema_version":1,"claimed_complete":True,"summary":"done","usage":{"cost_usd":-1}}))'
        self.assertEqual(self.trial(response=response)["status"], "protocol_error")

    def test_task_that_already_passes_is_invalid(self):
        self.grader.write_text('import json\nprint(json.dumps({"accepted":True,"reason":"always green"}))')
        self.assertEqual(self.trial()["status"], "invalid_task")

    def test_grader_bytes_are_frozen_before_agent_runs(self):
        suite = self.suite('Path("value.txt").write_text("correct")')
        self.grader.write_text('raise SystemExit("changed after load")')
        report = evaluate(suite, suite.tasks[0], "skill", self.root / "trials")
        self.assertEqual(report["status"], "accepted", report)

    def test_regression_failure_defeats_acceptance(self):
        self.config["tasks"][0]["regression"][0]["argv"] = [sys.executable, "-c",
            'from pathlib import Path; raise SystemExit(1 if Path("value.txt").read_text()=="correct" else 0)']
        report = self.trial('Path("value.txt").write_text("correct")')
        self.assertTrue(report["acceptance"]["accepted"])
        self.assertEqual(report["status"], "rejected")

    def test_empty_checks_unknown_fields_and_bad_paths_fail_admission(self):
        for raw in ({"argv": [], "timeout_seconds": 1},
                    {"argv": ["x"], "timeout_seconds": 0},
                    {"argv": ["x"], "timeout_seconds": 1, "typo": True}):
            with self.assertRaises(ValueError):
                Command.parse(raw)
        self.config["tasks"][0]["editable_files"] = ["../escape"]
        with self.assertRaises(ValueError):
            self.suite()


if __name__ == "__main__":
    unittest.main()
