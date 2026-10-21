"""Actual pinned GEPA execution with a scripted proposer; never calls a model."""
from pathlib import Path
import importlib.util
import json
import unittest
import experiments.tests.test_evaluator as fixtures
from experiments.evaluator.spec import Suite
from experiments.gepa.config import Experiment, disjoint
from experiments.gepa.finalize import compare
from experiments.gepa.optimize import optimize


class SplitTests(unittest.TestCase):
    def test_identical_task_cannot_enter_two_splits(self):
        fixture = fixtures.EvaluatorTests()
        fixture.setUp()
        try:
            task = fixture.suite().tasks[0]
            from dataclasses import replace
            with self.assertRaises(ValueError):
                disjoint([task], [replace(task, id="renamed")])
        finally:
            fixture.tearDown()


@unittest.skipUnless(importlib.util.find_spec("gepa"), "optional GEPA environment not installed")
class GepaIntegrationTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.EvaluatorTests()
        self.fixture.setUp()
        self.root = self.fixture.root
        self.fixture.suite('if "APPLY_FIX" in r["candidate_skill"]: Path("value.txt").write_text(r["prompt"])')
        for name in ("train", "validation", "holdout"):
            raw = json.loads(json.dumps(self.fixture.config))
            task = raw["tasks"][0]
            task["id"] = name
            task["prompt"] = name + "-target"
            grader = self.root / (name + "-grader.py")
            grader.write_text('from pathlib import Path\nimport json\n'
                              f'ok = Path("value.txt").read_text() == {task["prompt"]!r}\n'
                              'print(json.dumps({"accepted":ok,"reason":"expected target value"}))\n'
                              'raise SystemExit(0 if ok else 1)')
            task["grader"] = str(grader)
            (self.root / (name + ".json")).write_text(json.dumps(raw))
        (self.root / "seed.md").write_text("Inspect source; no fixture trigger.")
        config = {"schema_version": 1, "train_suite": "train.json", "validation_suite": "validation.json",
                  "seed_skill": "seed.md", "reflection_model": "unused-scripted-proposer",
                  "max_evals": 8, "max_proposals": 2, "admission_seconds": 60,
                  "max_reflection_cost_usd": 1.0}
        self.config_path = self.root / "experiment.json"
        self.config_path.write_text(json.dumps(config))

    def tearDown(self):
        self.fixture.tearDown()

    def test_real_gepa_search_and_separate_paired_final_evaluation(self):
        config = Experiment.load(self.config_path)
        output = self.root / "search"
        result = optimize(config, output, reflection=lambda messages: "```\nAPPLY_FIX\n```")
        self.assertTrue(result["complete"])
        self.assertLessEqual(len(result["evaluations"]), config.max_evals)
        self.assertIn("APPLY_FIX", (output / "selected-skill.md").read_text())
        self.assertTrue(all(r.get("task_id") != "holdout" for r in result["evaluations"]))
        final = compare(output / "selection.json", self.root / "holdout.json", self.root / "final", repeats=1)
        self.assertEqual(final["accepted"], {"seed": 0, "selected": 1})
        self.assertIsNone(result["agent_reported_cost_usd"])
        with self.assertRaises(ValueError):
            compare(output / "selection.json", self.root / "train.json", self.root / "leak", repeats=1)
        (output / "selected-skill.md").write_text("changed after selection")
        with self.assertRaises(ValueError):
            compare(output / "selection.json", self.root / "holdout.json", self.root / "tamper", repeats=1)


if __name__ == "__main__":
    unittest.main()
