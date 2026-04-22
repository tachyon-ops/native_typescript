import re
with open("test_results_all.txt") as f:
    lines = f.read().split("\n")

current_test = None
for line in lines:
    m = re.match(r"^\s*Running (?:unittests )?(?:src/.*|tests/([^ ]+)\.rs).*", line)
    if m:
        current_test = m.group(1) or line
    
    m2 = re.match(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;.*", line)
    if m2:
        res, passed, failed = m2.groups()
        print(f"{current_test}: {res} ({passed} passed, {failed} failed)")
