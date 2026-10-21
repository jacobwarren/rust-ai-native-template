from dataclasses import dataclass
from pathlib import Path
import json
import math
from experiments.evaluator.spec import Suite, digest, object_fields, positive, text


def task_key(task):
    # Deliberately exclude ID and prompt wording: renaming an identical task cannot create a holdout.
    return digest(json.dumps({"commit": task.commit,
                              "editable": task.editable_files,
                              "grader": digest(task.grader_bytes)}, sort_keys=True).encode())


def disjoint(left, right):
    if {t.id for t in left} & {t.id for t in right} or {task_key(t) for t in left} & {task_key(t) for t in right}:
        raise ValueError("task splits overlap; renaming an identical task does not make it independent")


@dataclass(frozen=True)
class Experiment:
    train: Suite
    validation: Suite
    seed: str
    model: str
    max_evals: int
    max_proposals: int
    admission_seconds: int
    max_reflection_cost_usd: float
    sha256: str

    @classmethod
    def load(cls, path):
        path = Path(path).resolve()
        data = path.read_bytes()
        raw = object_fields(json.loads(data), ["schema_version", "train_suite", "validation_suite",
            "seed_skill", "reflection_model", "max_evals", "max_proposals",
            "admission_seconds", "max_reflection_cost_usd"])
        if type(raw["schema_version"]) is not int or raw["schema_version"] != 1:
            raise ValueError("unsupported experiment schema")
        def local(key):
            return (path.parent / text(raw[key])).resolve(strict=True)
        train, validation = Suite.load(local("train_suite")), Suite.load(local("validation_suite"))
        disjoint(train.tasks, validation.tasks)
        if train.identity != validation.identity or train.agent != validation.agent:
            raise ValueError("freeze the same agent identity and command across splits")
        seed = local("seed_skill").read_text()
        if not seed.strip() or len(seed.encode()) > 32768:
            raise ValueError("seed must contain 1-32768 UTF-8 bytes")
        cost = raw["max_reflection_cost_usd"]
        if type(cost) not in {int, float} or not math.isfinite(cost) or cost <= 0:
            raise ValueError("reflection cost budget must be finite and positive")
        maximum = positive(raw["max_evals"])
        if maximum < len(validation.tasks) + 2:
            raise ValueError("evaluation budget is too small for baseline and a proposal")
        return cls(train, validation, seed, text(raw["reflection_model"]), maximum,
                   positive(raw["max_proposals"]), positive(raw["admission_seconds"]), cost, digest(data))
