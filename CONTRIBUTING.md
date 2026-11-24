# Contributing to LNX-victory

First of all, thank you for your interest in contributing to **LNX-victory**!

LNX-victory is an experimental, community-driven game compatibility and emulation project. Its goal is to improve compatibility, rendering, performance, resource handling, and overall execution through collaborative development.

Whether you're experienced with graphics programming, CPU emulation, shader translation, or simply interested in helping test and document the project, contributions are welcome.

We want to build this project together.

---

## 1. Getting Started

Before contributing:

1. Read the project's README.
2. Check existing issues and pull requests.
3. Look for an existing discussion related to your proposed change.
4. Fork the repository.
5. Clone your fork locally.
6. Create a separate branch for your changes.

We recommend using descriptive branch names:

- `feature/shader-cache`
- `feature/texture-optimization`
- `fix/memory-management`
- `fix/graphics-rendering`
- `docs/update-installation`

Avoid making unrelated changes in the same pull request.

## 2. Areas Where We Need Help

LNX-victory has several areas where contributions can make a difference.

### Graphics & Rendering
- Graphics API compatibility.
- Rendering correctness.
- Lighting and material handling.
- Geometry and texture rendering.
- GPU resource management.

### Shader Translation
- Shader translation improvements.
- Compilation performance.
- Persistent shader caching.
- Cache invalidation and reliability.
- Shader debugging and validation.

### Texture & Resource Management
- Texture decoding and handling.
- Texture caching.
- Memory allocation improvements.
- Resource lifetime management.
- Asset streaming and loading performance.

### CPU & System Emulation
- Instruction translation.
- CPU compatibility.
- Thread scheduling.
- Synchronization.
- Memory management.

### Performance & Optimization
- CPU overhead reduction.
- GPU workload optimization.
- Memory transfer optimization.
- Frame-time improvements.
- Profiling and benchmarking.

### Testing & Documentation
- Reproducing bugs.
- Reporting compatibility issues.
- Improving documentation.
- Testing different hardware configurations.
- Providing reproducible performance measurements.

You don't need to work on every subsystem. Focus on the area you're most comfortable with.

## 3. Development Guidelines

Please follow these principles when modifying the project:

- Keep changes modular and maintainable.
- Avoid unnecessary dependencies.
- Preserve existing functionality whenever possible.
- Avoid introducing unrelated changes.
- Handle errors and unexpected inputs properly.
- Document complex or non-obvious implementations.
- Prefer measurable optimizations over assumptions.
- Avoid hardcoding hardware-specific behavior unless explicitly required.

Performance optimizations should ideally include measurements before and after the change.

When modifying caches, consider:
- Cache invalidation.
- Version compatibility.
- Corrupted or incomplete cache entries.
- Concurrent access.
- Persistent storage behavior.
- GPU and driver compatibility.

A faster implementation is only useful if it remains correct and reliable.

## 4. Pull Request Guidelines

Before submitting a pull request:

- Make sure your changes compile.
- Test the affected functionality.
- Check for regressions.
- Remove temporary debugging code.
- Include relevant logs or benchmarks when useful.
- Explain any known limitations.

Every pull request should include:

**What changed?**

Describe the modifications.

**Why was it changed?**

Explain the problem or improvement.

**How was it tested?**

Include your testing environment and results.

**Are there any known limitations?**

Mention incomplete functionality, compatibility concerns, or potential regressions.

Keep pull requests focused. Large architectural changes should be discussed before implementation whenever possible.

## 5. Commit Messages

Use clear and descriptive commit messages.

Examples:

- `fix: resolve shader cache invalidation`
- `feat: add persistent texture cache`
- `perf: reduce unnecessary memory transfers`
- `refactor: improve resource management`
- `docs: update build instructions`

Avoid vague messages such as `update`, `changes`, or `fix stuff`.

## 6. Project Content & Copyright

LNX-victory is an independent, unofficial compatibility and emulation project.

**Do not submit or upload:**
- Proprietary game executables.
- Copyrighted game assets or extracted game resources.
- Leaked or otherwise unauthorized game files.
- Third-party proprietary source code without permission.
- Content that you do not have the right to distribute.

Contributions should consist of original code, documentation, independently created tools, and other materials that you are authorized to share.

Do not use pull requests or issues to distribute copyrighted game files.

Please respect the licenses of all third-party libraries and dependencies.

## 7. Bug Reports

When reporting a bug, provide as much useful information as possible.

Include:

- Operating system.
- CPU and GPU.
- GPU driver version.
- Project version or commit hash.
- Steps to reproduce the issue.
- Expected behavior.
- Actual behavior.
- Relevant logs or screenshots.

For performance-related issues, include:
- Average FPS.
- Frame times, if available.
- CPU and GPU utilization.
- VRAM and system RAM usage.
- Relevant testing conditions.

Please avoid submitting reports that only say something is broken without explaining what happened.

## 8. Discussions & Community

Be respectful toward other contributors.

We expect everyone to:
- Treat other developers with respect.
- Provide constructive feedback.
- Accept that technical disagreements happen.
- Avoid personal attacks and harassment.
- Give credit where credit is due.
- Help newcomers when possible.

Different approaches are welcome. Technical discussions should focus on evidence, implementation details, and measurable results.

## 9. Questions & Suggestions

If you have an idea, optimization, bug report, or architectural proposal, feel free to open an issue or start a discussion.

For major changes, discussing the approach before submitting a large pull request can save everyone time.

Not every suggestion will be accepted, but thoughtful contributions and feedback are appreciated.

---

## 10. Final Words

LNX-victory is an ongoing experimental project. Compatibility, rendering, and performance are continuously evolving, and some features may remain incomplete or unstable.

There is still a lot of work to do.

Every useful contribution helps move the project forward, whether it's a major graphics improvement, a small bug fix, better documentation, or a reproducible test case.

**Thank you for helping build LNX-victory.**

Let's make it better, together.

---

*LNX-victory — Community-driven development.*
