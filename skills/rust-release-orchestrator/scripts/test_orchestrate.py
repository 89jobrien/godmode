import json
import unittest
from pathlib import Path
from unittest.mock import patch

import orchestrate


class ComputePublishOrderTests(unittest.TestCase):
    def test_non_empty_registry_allowlist_is_publishable(self):
        packages = [
            {
                "id": "public 1.0.0",
                "name": "public",
                "publish": None,
                "dependencies": [],
            },
            {
                "id": "allowlisted 1.0.0",
                "name": "allowlisted",
                "publish": ["company-registry"],
                "dependencies": [],
            },
            {
                "id": "private 1.0.0",
                "name": "private",
                "publish": [],
                "dependencies": [],
            },
        ]
        metadata = {
            "packages": packages,
            "workspace_members": [package["id"] for package in packages],
        }

        with patch.object(orchestrate, "run", return_value=(json.dumps(metadata), 0)):
            order = orchestrate.compute_publish_order(Path("/workspace"))

        self.assertCountEqual(
            [package["name"] for package in order], ["public", "allowlisted"]
        )


if __name__ == "__main__":
    unittest.main()
