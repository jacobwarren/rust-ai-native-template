"""Source snapshots and controlled changes, independent of agent Git metadata."""
from pathlib import Path
import io
import os
import stat
import subprocess
import tarfile
from .spec import digest, relative


def git(repo, *args):
    return subprocess.run(["git", "-c", "core.hooksPath=/dev/null", "-C", str(repo), *args],
                          check=True, capture_output=True, timeout=60).stdout


def snapshot(repo, commit, destination):
    if git(repo, "rev-parse", "--verify", f"{commit}^{{commit}}").decode().strip() != commit:
        raise ValueError("task commit did not resolve exactly")
    destination.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(git(repo, "archive", "--format=tar", commit))) as archive:
        for member in archive.getmembers():
            name = relative(member.name.rstrip("/"))
            path = destination / name
            if member.isdir():
                path.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                path.parent.mkdir(parents=True, exist_ok=True)
                with archive.extractfile(member) as source:
                    path.write_bytes(source.read())
                path.chmod(0o755 if member.mode & 0o111 else 0o644)
            else:
                raise ValueError("task snapshots do not support symlinks, submodules, or special files")
    # Explicitly reject gitlinks, which git archive can omit silently.
    if any(line.startswith(b"160000 ") for line in git(repo, "ls-tree", "-r", commit).splitlines()):
        raise ValueError("submodules require an explicit evaluator extension")


def inventory(root):
    """Hash contents + executable bit; never follow an agent-created symlink."""
    result = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        rel_dir = Path(directory).relative_to(root)
        if rel_dir == Path("."):
            dirs[:] = [d for d in dirs if d not in {".git", "target"}]
        for name in dirs + files:
            path = Path(directory) / name
            mode = path.lstat().st_mode
            if stat.S_ISLNK(mode) or not (stat.S_ISREG(mode) or stat.S_ISDIR(mode)):
                raise ValueError(f"unsupported source entry: {path.relative_to(root)}")
        for name in files:
            path = Path(directory) / name
            if rel_dir == Path(".") and name == ".git":
                continue
            data = path.read_bytes()
            result[path.relative_to(root).as_posix()] = (digest(data), bool(path.stat().st_mode & 0o111))
    return result


def tree_digest(items):
    import json
    return digest(json.dumps(items, sort_keys=True, separators=(",", ":")).encode())


def changed_files(before, after):
    return sorted(name for name in before.keys() | after.keys() if before.get(name) != after.get(name))


def initialize_git(root):
    git(root, "init", "--quiet")
    git(root, "add", "--force", ".")
    git(root, "-c", "user.name=Evaluator", "-c", "user.email=evaluator@example.invalid",
        "-c", "commit.gpgsign=false", "commit", "--quiet", "-m", "Frozen evaluation snapshot")
