# Trend calculation

Read this before comparing a current health measurement to the previous one, or before assigning trend symbols on a scorecard.

## Trend Calculation

For each metric, compare current value to the most recent previous value:

### Improvement threshold (5% change)

- **test_count**: 5% increase is "better"
- **clippy_warnings**: 5% decrease is "better"
- **todo_density**: 5% decrease is "better"
- **api_surface**: 5% change (either direction) noted; growth is neutral, shrink is slight improvement
- **dependency_count**: 5% decrease is "better"
- **avg_module_lines**: 5% decrease is "better"
- **doc_coverage**: 5% increase is "better"

### Trend symbols

- ↑ = **better** (improved by >5%)
- → = **stable** (changed <5%)
- ↓ = **worse** (degraded by >5%)
