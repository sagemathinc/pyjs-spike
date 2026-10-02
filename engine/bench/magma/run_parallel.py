"""run_parallel.py FROM TO JOBS: Magma's newforms.m on levels FROM..TO split
over JOBS processes (process i takes FROM+i, FROM+i+JOBS, ...).  Prints the
total rational newforms, wall time, summed CPU time and the largest peak
RSS of any process."""
import os, re, resource, subprocess, sys, time

lo, hi, k = map(int, sys.argv[1:4])
script = os.path.join(os.path.dirname(os.path.abspath(__file__)), "newforms.m")
magma = os.path.expanduser("~/bin/magma")
t = time.time()
procs = [subprocess.Popen([magma, "-b", f"a:={lo + i}", f"b:={hi}", f"k:={k}", script], stdout=subprocess.PIPE, text=True) for i in range(k)]
outs = [p.communicate()[0] for p in procs]
wall = time.time() - t
forms = cpu = 0.0
for o in outs:
    m = re.search(r"^levels .*: (\d+) rational newforms, ([\d.]+) s CPU", o, re.M)
    forms += int(m.group(1))
    cpu += float(m.group(2))
rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024
print(f"magma levels {lo}..{hi} on {k} processes: {int(forms)} rational newforms, wall {wall:.1f} s, CPU {cpu:.1f} s, largest process peak RSS {rss:.0f} MB")
