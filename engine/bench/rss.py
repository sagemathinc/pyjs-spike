"""rss.py CMD...: run a command, then print its wall time and peak RSS."""
import resource, subprocess, sys, time
t = time.time()
subprocess.run(sys.argv[1:])
print(f"wall {time.time() - t:.2f} s, peak RSS {resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024:.0f} MB", file=sys.stderr)
