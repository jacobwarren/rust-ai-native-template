"""Strict wire parsing into immutable experiment specifications."""
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
import hashlib
import json
import re


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def object_fields(value, required, optional=()):
    if not isinstance(value, dict) or set(value) - set(required) - set(optional) or set(required) - set(value):
        raise ValueError(f"expected fields {required}; optional {optional}")
    return value


def text(value):
    if not isinstance(value, str) or not value.strip() or "\0" in value:
        raise ValueError("expected nonempty text without NUL")
    return value


def positive(value):
    if type(value) is not int or value < 1:
        raise ValueError("expected a positive integer")
    return value


def identifier(value):
    if not re.fullmatch(r"[a-zA-Z0-9_-]{1,80}", text(value)):
        raise ValueError("invalid identifier")
    return value


def relative(value):
    value = text(value)
    p = PurePosixPath(value)
    if p.is_absolute() or ".." in p.parts or "\\" in value or str(p) != value or value == ".":
        raise ValueError("expected normalized relative file path")
    if p.parts[0] in {".git", "target"}:
        raise ValueError("runtime/control directories cannot be editable source")
    return value


@dataclass(frozen=True)
class Command:
    argv: tuple[str, ...]
    timeout_seconds: int

    @classmethod
    def parse(cls, raw):
        object_fields(raw, ["argv", "timeout_seconds"])
        argv = raw["argv"]
        if not isinstance(argv, list) or not argv:
            raise ValueError("command requires argv")
        text(argv[0])
        if any(not isinstance(a, str) or "\0" in a for a in argv):
            raise ValueError("invalid command argument")
        return cls(tuple(argv), positive(raw["timeout_seconds"]))


@dataclass(frozen=True)
class Task:
    id: str
    repo: Path
    commit: str
    prompt: str
    editable_files: tuple[str, ...]
    regression: tuple[Command, ...]
    grader_bytes: bytes
    grader_timeout_seconds: int

    @classmethod
    def parse(cls, raw):
        object_fields(raw, ["id", "repo", "commit", "prompt", "editable_files",
                            "regression", "grader", "grader_timeout_seconds"])
        repo = Path(text(raw["repo"]))
        grader = Path(text(raw["grader"]))
        if not repo.is_absolute() or not grader.is_absolute():
            raise ValueError("repository and grader paths must be absolute")
        repo, grader = repo.resolve(strict=True), grader.resolve(strict=True)
        if not repo.is_dir() or not grader.is_file() or grader.is_relative_to(repo):
            raise ValueError("grader must be a separate file outside the task repository")
        commit = text(raw["commit"])
        if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit):
            raise ValueError("pin an exact full Git commit hash")
        allowed, checks = raw["editable_files"], raw["regression"]
        if not isinstance(allowed, list) or not allowed or not isinstance(checks, list) or not checks:
            raise ValueError("editable_files and regression must be nonempty lists")
        allowed = tuple(relative(p) for p in allowed)
        if len(set(allowed)) != len(allowed):
            raise ValueError("duplicate editable file")
        return cls(identifier(raw["id"]), repo, commit, text(raw["prompt"]), allowed,
                   tuple(Command.parse(c) for c in checks), grader.read_bytes(),
                   positive(raw["grader_timeout_seconds"]))


@dataclass(frozen=True)
class Suite:
    identity: str
    agent: Command
    tasks: tuple[Task, ...]
    source_sha256: str

    @classmethod
    def load(cls, path):
        data = Path(path).read_bytes()
        raw = object_fields(json.loads(data), ["schema_version", "agent_identity", "agent", "tasks"])
        if raw["schema_version"] != 1 or type(raw["schema_version"]) is not int:
            raise ValueError("unsupported suite version")
        if not isinstance(raw["tasks"], list) or not raw["tasks"]:
            raise ValueError("suite requires tasks")
        tasks = tuple(Task.parse(t) for t in raw["tasks"])
        if len({t.id for t in tasks}) != len(tasks):
            raise ValueError("duplicate task IDs")
        return cls(text(raw["agent_identity"]), Command.parse(raw["agent"]), tasks, digest(data))
