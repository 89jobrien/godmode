import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/orchestrate.rs"


class ReleaseOrchestratorIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        built = subprocess.run(["rust-script", "--force", str(SCRIPT), "--help"], text=True, capture_output=True, timeout=300)
        if built.returncode != 0:
            raise RuntimeError(built.stdout + built.stderr)

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.workspace = self.root / "workspace"
        (self.workspace / "core").mkdir(parents=True)
        (self.workspace / "adapter").mkdir()
        for crate in ("core", "adapter"):
            (self.workspace / crate / "Cargo.toml").write_text("[package]\nname=\"x\"\nversion=\"0.1.0\"\n")
        self.log = self.root / "cargo.log"
        fake = self.bin / "cargo"
        fake.write_text("""#!/usr/bin/env python3
import os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ[\"FAKE_CARGO_LOG\"])
with log.open(\"a\") as handle: handle.write(pathlib.Path.cwd().name + \"|\" + \" \".join(args) + \"\\n\")
root = pathlib.Path(os.environ[\"FAKE_WORKSPACE\"])
if args and args[0] == \"metadata\":
    print(os.environ[\"FAKE_METADATA\"]); raise SystemExit(0)
if args[:3] == [\"fmt\", \"--all\", \"--check\"]:
    raise SystemExit(0 if (root / \".formatted\").exists() else 1)
if args[:2] == [\"fmt\", \"--all\"]:
    (root / \".formatted\").write_text(\"yes\"); raise SystemExit(0)
if args and args[0] == \"clippy\" and \"--fix\" in args:
    (root / \".clippy-fixed\").write_text(\"yes\"); raise SystemExit(0)
if args and args[0] == \"clippy\":
    raise SystemExit(0 if (root / \".clippy-fixed\").exists() else 1)
if args and args[0] == \"publish\" and pathlib.Path.cwd().name == \"adapter\":
    if os.environ.get(\"FAIL_ADAPTER\") == \"1\":
        print(\"HTTP 429 Too Many Requests\", file=sys.stderr); raise SystemExit(1)
    once = root / \".adapter-failed-once\"
    if os.environ.get(\"FAIL_ADAPTER_ONCE\") == \"1\" and not once.exists():
        once.write_text(\"yes\")
        print(\"HTTP 429 Too Many Requests\", file=sys.stderr); raise SystemExit(1)
raise SystemExit(0)
""")
        fake.chmod(0o755)
        packages = []
        for name, deps in (("core", []), ("adapter", [{"name": "core"}])):
            packages.append({"id": name, "name": name, "version": "0.1.0", "manifest_path": str(self.workspace / name / "Cargo.toml"), "publish": None, "dependencies": deps})
        self.env = dict(os.environ)
        self.env.update({
            "PATH": str(self.bin) + os.pathsep + os.environ.get("PATH", ""),
            "FAKE_CARGO_LOG": str(self.log),
            "FAKE_WORKSPACE": str(self.workspace),
            "FAKE_METADATA": json.dumps({"packages": packages, "workspace_members": ["core", "adapter"]}),
            "GODMODE_RELEASE_MAX_RETRIES": "1",
            "GODMODE_RELEASE_RETRY_BASE_MS": "0",
        })

    def tearDown(self):
        self.tmp.cleanup()

    def run_release(self, *args, fail_adapter=False, fail_adapter_once=False, max_retries=None):
        env = dict(self.env)
        if fail_adapter: env["FAIL_ADAPTER"] = "1"
        if fail_adapter_once: env["FAIL_ADAPTER_ONCE"] = "1"
        if max_retries is not None: env["GODMODE_RELEASE_MAX_RETRIES"] = str(max_retries)
        return subprocess.run(["rust-script", str(SCRIPT), "--workspace", str(self.workspace), *args], env=env, text=True, capture_output=True, timeout=180)

    def test_mutation_failure_state_resume_report_and_cleanup(self):
        failed = self.run_release(fail_adapter=True)
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn("RELEASE REPORT", failed.stdout)
        self.assertIn("failed", failed.stdout)
        state_path = self.workspace / ".release-state.json"
        state = json.loads(state_path.read_text())
        self.assertEqual(state, {"published": ["core"], "gates_passed": True})
        self.assertTrue((self.workspace / ".formatted").exists())
        self.assertTrue((self.workspace / ".clippy-fixed").exists())

        resumed = self.run_release("--resume")
        self.assertEqual(resumed.returncode, 0, resumed.stdout + resumed.stderr)
        self.assertIn("skipped", resumed.stdout)
        self.assertFalse(state_path.exists())
        lines = self.log.read_text().splitlines()
        self.assertEqual(sum(line == "core|publish" for line in lines), 1)
        self.assertEqual(sum(line == "adapter|publish" for line in lines), 2)


    def test_rate_limit_retries_then_cleans_up_successful_state(self):
        result = self.run_release(fail_adapter_once=True, max_retries=2)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("Rate limited; retrying", result.stdout)
        self.assertFalse((self.workspace / ".release-state.json").exists())
        lines = self.log.read_text().splitlines()
        self.assertEqual(sum(line == "adapter|publish" for line in lines), 2)


if __name__ == "__main__":
    unittest.main()
