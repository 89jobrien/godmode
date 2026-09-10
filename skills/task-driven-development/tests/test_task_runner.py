import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "skills/task-driven-development/helpers/task-runner.rs"
FIXTURES = Path(__file__).parent / "fixtures"


class TaskRunnerExecutableTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        built = subprocess.run(["rust-script", "--force", str(RUNNER), "--help"], text=True, capture_output=True, timeout=300)
        if built.returncode != 0:
            raise RuntimeError(built.stdout + built.stderr)

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.cwd = Path(self.tmp.name)
        self.bin = self.cwd / "bin"
        self.bin.mkdir()
        cargo = self.bin / "cargo"
        cargo.write_text("""#!/bin/sh
exit ${FAKE_CARGO_EXIT:-0}
""")
        cargo.chmod(0o755)
        self.env = dict(os.environ)
        self.env["PATH"] = str(self.bin) + os.pathsep + os.environ.get("PATH", "")

    def tearDown(self):
        self.tmp.cleanup()

    def run_runner(self, *args):
        return subprocess.run(
            ["rust-script", str(RUNNER), *args], cwd=self.cwd,
            env=self.env, text=True, capture_output=True, timeout=120,
        )

    def install_fixture(self, name):
        shutil.copy(FIXTURES / name, self.cwd / "tdd-tasks.yaml")

    def test_rejects_red_transition_from_green_without_mutating_fixture(self):
        self.install_fixture("green.yaml")
        before = (self.cwd / "tdd-tasks.yaml").read_text()
        result = self.run_runner("red", "t1")
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("must be pending", result.stderr)
        self.assertEqual((self.cwd / "tdd-tasks.yaml").read_text(), before)

    def test_missing_state_and_unknown_task_report_actionable_errors(self):
        missing = self.run_runner("status")
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("run: task-runner.rs init", missing.stderr)
        self.install_fixture("green.yaml")
        unknown = self.run_runner("green", "missing")
        self.assertNotEqual(unknown.returncode, 0)
        self.assertIn("not found", unknown.stderr)
        shutil.copy(FIXTURES / "malformed.txt", self.cwd / "tdd-tasks.yaml")
        malformed = self.run_runner("status")
        self.assertNotEqual(malformed.returncode, 0)
        self.assertIn("parsing tdd-tasks.yaml", malformed.stderr)


    def test_fixture_chain_advances_red_green_refactor_and_unblocks_next(self):
        self.install_fixture("pending-chain.yaml")
        blocked = self.run_runner("red", "t2")
        self.assertNotEqual(blocked.returncode, 0)
        self.assertIn("blocked", blocked.stderr)
        for args in (("red", "t1"), ("green", "t1"), ("refactor", "t1")):
            result = self.run_runner(*args)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        state = (self.cwd / "tdd-tasks.yaml").read_text()
        self.assertIn("phase: done", state)
        self.assertIn("status: done", state)
        next_task = self.run_runner("next")
        self.assertIn("t2: dependent behavior", next_task.stdout)

    def test_failed_cargo_keeps_red_state_for_retry(self):
        self.install_fixture("pending-chain.yaml")
        self.assertEqual(self.run_runner("red", "t1").returncode, 0)
        self.env["FAKE_CARGO_EXIT"] = "1"
        failed = self.run_runner("green", "t1")
        self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)
        state = (self.cwd / "tdd-tasks.yaml").read_text()
        self.assertIn("phase: red", state)
        self.assertIn("status: running", state)


if __name__ == "__main__":
    unittest.main()
