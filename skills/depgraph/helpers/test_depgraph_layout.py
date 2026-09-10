import unittest

from depgraph_layout import radial_positions, reduction_percent, ring_index


class RadialPositionsTests(unittest.TestCase):
    def test_empty_and_singleton_layouts_are_stable(self):
        self.assertEqual(radial_positions([], {}, 10, 20, 5), {})
        self.assertEqual(radial_positions(["only"], {"only": 7}, 10, 20, 5), {7: (10, 20)})

    def test_multiple_zero_ring_crates_receive_distinct_positions(self):
        positions = radial_positions(
            ["domain-a", "domain-b", "domain-c"],
            {"domain-a": "a", "domain-b": "b", "domain-c": "c"},
            100,
            100,
            60,
        )

        self.assertEqual(set(positions), {"a", "b", "c"})
        self.assertEqual(len(set(positions.values())), 3)


class ReductionPercentTests(unittest.TestCase):
    def test_zero_edges_returns_zero_percent(self):
        self.assertEqual(reduction_percent(0, 0), 0)

    def test_reports_removed_edge_percentage(self):
        self.assertEqual(reduction_percent(4, 3), 25)

    def test_rejects_impossible_edge_boundaries(self):
        with self.assertRaises(ValueError):
            reduction_percent(1, 2)


class RingIndexTests(unittest.TestCase):
    def test_shallow_layers_remain_adjacent(self):
        self.assertEqual(ring_index(0), 0)
        self.assertEqual(ring_index(1), 1)

    def test_deep_layers_are_capped_at_outer_ring(self):
        self.assertEqual(ring_index(8), 3)

    def test_rejects_negative_layers_and_empty_ring_sets(self):
        with self.assertRaises(ValueError):
            ring_index(-1)
        with self.assertRaises(ValueError):
            ring_index(0, 0)


if __name__ == "__main__":
    unittest.main()
