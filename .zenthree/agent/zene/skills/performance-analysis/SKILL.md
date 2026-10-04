---
name: performance-analysis
description: Measure and improve software performance using evidence and defined targets.
version: "1.0.0"
tags:
  - performance
  - profiling
  - optimization
---
# Performance Analysis

## Purpose

Use this Skill when investigating latency, throughput, memory, startup, or resource usage.

## Workflow

1. Define the affected operation, workload, environment, and measurable target.
2. Establish a representative baseline.
3. Use the workspace's available profiler, benchmark, tracing, or measurement tools.
4. Identify the dominant bottleneck from evidence.
5. Apply the smallest change addressing that bottleneck.
6. Re-measure under the same conditions and check correctness regressions.

## Rules

- Measure before and after; do not optimize by intuition alone.
- Do not trade correctness or security for unmeasured gains.
- Keep workloads and environments comparable.
- Report uncertainty, variance, and limitations.
