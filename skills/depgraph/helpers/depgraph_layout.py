import math


def radial_positions(crate_names, name_to_id, cx, cy, radius):
    count = len(crate_names)
    if count == 0:
        return {}
    if count == 1:
        return {name_to_id[crate_names[0]]: (cx, cy)}

    return {
        name_to_id[name]: (
            cx + radius * math.cos(math.radians(-90 + (360 / count) * index)),
            cy + radius * math.sin(math.radians(-90 + (360 / count) * index)),
        )
        for index, name in enumerate(crate_names)
    }


def reduction_percent(edge_count, reduced_count):
    if edge_count < 0 or reduced_count < 0 or reduced_count > edge_count:
        raise ValueError("edge counts must satisfy 0 <= reduced <= total")
    if edge_count == 0:
        return 0
    return 100 * (edge_count - reduced_count) // edge_count


def ring_index(layer, ring_count=4):
    if layer < 0 or ring_count < 1:
        raise ValueError("layer must be non-negative and ring_count must be positive")
    return min(layer, ring_count - 1)
