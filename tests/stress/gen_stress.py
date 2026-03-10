#!/usr/bin/env python3
import os

def gen_deep_dependency(n=1000):
    with open("tests/stress/deep_deps.lum", "w") as f:
        f.write("entity Chain {\n")
        f.write("  base: Number\n")
        f.write("  v0 := base + 1\n")
        for i in range(1, n):
            f.write(f"  v{i} := v{i-1} + 1\n")
        f.write("}\n\n")
        f.write("let Chain = Chain { base: 0 }\n")
        f.write(f"show \"Final value: {{Chain.v{n-1}}}\"\n")

def gen_rule_cascade(n=100):
    with open("tests/stress/rule_cascade.lum", "w") as f:
        f.write("entity Cascade {\n")
        for i in range(n + 1):
            f.write(f"  f{i}: Boolean\n")
        f.write("}\n\n")
        f.write("let Cascade = Cascade {\n")
        for i in range(n + 1):
            f.write(f"  f{i}: false" + ("," if i < n else "") + "\n")
        f.write("}\n\n")
        for i in range(n):
            f.write(f"rule \"r{i}\" {{\n")
            f.write(f"  when Cascade.f{i} becomes true\n")
            f.write(f"  then update Cascade.f{i+1} to true\n")
            f.write("}\n\n")
        f.write("update Cascade.f0 to true\n")
        f.write(f"show \"Cascade finished: {{Cascade.f{n}}}\"\n")

def gen_heavy_timers(n=10000):
    with open("tests/stress/heavy_timers.lum", "w") as f:
        f.write("entity Tick { count: Number }\n")
        f.write("let Tick = Tick { count: 0 }\n\n")
        for i in range(n):
            f.write(f"rule \"t{i}\" {{\n")
            f.write(f"  every {i+1} s\n")
            f.write(f"  then show \"Tick {i}\"\n")
            f.write("}\n\n")

if __name__ == "__main__":
    os.makedirs("tests/stress", exist_ok=True)
    gen_deep_dependency(1000)
    gen_rule_cascade(110) # 110 should trigger MAX_DEPTH (100)
    gen_heavy_timers(10000)
    print("Stress tests generated in tests/stress/")
