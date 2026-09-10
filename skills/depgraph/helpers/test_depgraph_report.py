import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HELPERS = Path(__file__).parent


def load_module():
    sys.path.insert(0, str(HELPERS))
    spec = importlib.util.spec_from_file_location("depgraph_report", HELPERS / "depgraph-report.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DepgraphReportTests(unittest.TestCase):
    def setUp(self):
        self.mod = load_module()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_import_has_no_cli_or_filesystem_side_effects(self):
        self.assertTrue(callable(self.mod.main))
        self.assertFalse((self.root / "target").exists())

    def test_safe_read_and_latest_sarif_ignore_corruption_and_count_levels(self):
        bad = self.root / ".ctx" / "bad.sarif"
        bad.parent.mkdir()
        bad.write_text("{")
        self.assertIsNone(self.mod._safe_read_json(bad))
        good = self.root / "target" / "latest.sarif"
        good.parent.mkdir()
        good.write_text(json.dumps({"runs": [{"tool": {"driver": {"name": "scan<&"}}, "results": [{"level": "error"}, {"level": "warning"}, {}]}]}))
        os.utime(good, (bad.stat().st_mtime + 2, bad.stat().st_mtime + 2))
        meta = self.mod._latest_sarif_meta(str(self.root))
        self.assertEqual((meta["results"], meta["errors"], meta["warnings"], meta["notes"]), (3, 1, 1, 1))
        html = self.mod._sarif_insight_html(meta)
        self.assertIn("scan&lt;&amp;", html)
        self.assertNotIn("scan<&", html)

    def test_snapshots_and_history_are_written(self):
        report = self.root / "out" / "report.html"
        report.parent.mkdir()
        report.write_text("report")
        item = {"severity": "high", "src": "a", "dst": "b", "src_ring": "App", "dst_ring": "Core", "remedy": "use facade"}
        self.mod._write_action_snapshots(report, [item], crate_count=2, good_deps=0, bad_deps=1, arch_health_percent=0)
        copied = self.mod._write_history_artifacts(report, self.root / "history", keep_timestamp=False)
        self.assertTrue((report.parent / "depgraph-action-items.json").exists())
        self.assertTrue((report.parent / "depgraph-action-items.csv").exists())
        self.assertEqual(len(copied), 4)

    def test_subprocess_errors_include_missing_command_and_stderr(self):
        with self.assertRaisesRegex(RuntimeError, "required command not found"):
            self.mod.run(["definitely-not-a-command"])
        with patch.object(subprocess, "run", return_value=subprocess.CompletedProcess(["x"], 2, "", "boom")):
            with self.assertRaisesRegex(RuntimeError, "x failed: boom"):
                self.mod.run(["x"])


    def fake_run(self, dot):
        def run(cmd, **kwargs):
            if cmd[:2] == ["cargo", "depgraph"]:
                return dot
            if "rev-parse" in cmd:
                return "feature/<unsafe>"
            if "merge-base" in cmd:
                return "abc123"
            if cmd[-3:-1] == ["--format=%as", "abc123"] or "--format=%as" in cmd:
                return "2026-01-01"
            if "--format=%h|%as|%s" in cmd:
                return "def456|2026-01-02|fix: <script>alert(1)</script>"
            raise AssertionError(cmd)
        return run

    def test_singleton_end_to_end_escapes_graph_git_and_branch_content(self):
        output = self.root / "report.html"
        dot = """digraph {
  0 [label = "core<&"]
}"""
        with patch.object(self.mod, "run", side_effect=self.fake_run(dot)):
            self.mod.main(["--repo", str(self.root), "--output", str(output), "--base", "main<&"])
        html = output.read_text()
        self.assertIn("core&lt;&amp;", html)
        self.assertIn("feature/&lt;unsafe&gt;", html)
        self.assertIn("&lt;script&gt;alert(1)&lt;/script&gt;", html)
        self.assertNotIn("<script>alert(1)</script>", html)
        self.assertIn('1</div><div class="l">Crates', html)

    def test_empty_graph_generates_report_and_cycle_is_actionable(self):
        output = self.root / "empty.html"
        with patch.object(self.mod, "run", side_effect=self.fake_run("digraph {\n}")):
            self.mod.main(["--repo", str(self.root), "--output", str(output)])
        html = output.read_text()
        self.assertIn("none", html)
        self.assertIn('0</div><div class="l">Crates', html)
        cycle = """digraph {
  0 [label = "a"]
  1 [label = "b"]
  0 -> 1 []
  1 -> 0 []
}"""
        with patch.object(self.mod, "run", side_effect=self.fake_run(cycle)):
            with self.assertRaisesRegex(RuntimeError, "cycle detected.*a, b"):
                self.mod.main(["--repo", str(self.root), "--output", str(self.root / "cycle.html")])

    def test_generation_propagates_cargo_depgraph_failure(self):
        with patch.object(self.mod, "run", side_effect=RuntimeError("cargo depgraph failed: broken")):
            with self.assertRaisesRegex(RuntimeError, "broken"):
                self.mod.main(["--repo", str(self.root), "--output", str(self.root / "failed.html")])


if __name__ == "__main__":
    unittest.main()
