"""itertools, following the pure-Python equivalents in CPython's docs."""


def count(start=0, step=1):
    n = start
    while True:
        yield n
        n += step


def cycle(iterable):
    saved = []
    for element in iterable:
        yield element
        saved.append(element)
    while saved:
        for element in saved:
            yield element


def repeat(obj, times=None):
    if times is None:
        while True:
            yield obj
    else:
        for i in range(times):
            yield obj


def accumulate(iterable, func=None, *, initial=None):
    it = iter(iterable)
    total = initial
    if initial is None:
        try:
            total = next(it)
        except StopIteration:
            return
    yield total
    for element in it:
        total = total + element if func is None else func(total, element)
        yield total


class chain:
    def __init__(self, *iterables):
        self._gen = self._run(iterables)

    @staticmethod
    def _run(iterables):
        for it in iterables:
            for element in it:
                yield element

    @classmethod
    def from_iterable(cls, iterables):
        c = cls()
        c._gen = cls._run(iterables)
        return c

    def __iter__(self):
        return self

    def __next__(self):
        return next(self._gen)


def compress(data, selectors):
    return (d for d, s in zip(data, selectors) if s)


def dropwhile(predicate, iterable):
    it = iter(iterable)
    for x in it:
        if not predicate(x):
            yield x
            break
    for x in it:
        yield x


def takewhile(predicate, iterable):
    for x in iterable:
        if predicate(x):
            yield x
        else:
            break


def filterfalse(predicate, iterable):
    if predicate is None:
        predicate = bool
    for x in iterable:
        if not predicate(x):
            yield x


def groupby(iterable, key=None):
    keyfunc = (lambda x: x) if key is None else key
    it = iter(iterable)
    exhausted = False
    try:
        curr_value = next(it)
    except StopIteration:
        return
    curr_key = keyfunc(curr_value)

    def grouper(target_key):
        nonlocal curr_value, curr_key, exhausted
        yield curr_value
        for curr_value in it:
            curr_key = keyfunc(curr_value)
            if curr_key != target_key:
                return
            yield curr_value
        exhausted = True

    while not exhausted:
        target_key = curr_key
        curr_group = grouper(target_key)
        yield curr_key, curr_group
        if curr_key == target_key:
            for _ in curr_group:
                pass


def islice(iterable, *args):
    s = slice(*args)
    start, stop, step = s.start or 0, s.stop, s.step or 1
    if start < 0 or (stop is not None and stop < 0) or step <= 0:
        raise ValueError("Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize.")
    it = iter(iterable)
    i = 0
    nexti = start
    while stop is None or nexti < stop:
        try:
            v = next(it)
        except StopIteration:
            return
        if i == nexti:
            yield v
            nexti += step
        i += 1


def starmap(function, iterable):
    for args in iterable:
        yield function(*args)


def tee(iterable, n=2):
    it = iter(iterable)
    buffers = [[] for _ in range(n)]

    def gen(mybuf):
        while True:
            if not mybuf:
                try:
                    v = next(it)
                except StopIteration:
                    return
                for b in buffers:
                    b.append(v)
            yield mybuf.pop(0)

    return tuple(gen(b) for b in buffers)


def zip_longest(*args, fillvalue=None):
    iterators = [iter(it) for it in args]
    num_active = len(iterators)
    if not num_active:
        return
    while True:
        values = []
        for i, it in enumerate(iterators):
            try:
                value = next(it)
            except StopIteration:
                num_active -= 1
                if not num_active:
                    return
                iterators[i] = repeat(fillvalue)
                value = fillvalue
            values.append(value)
        yield tuple(values)


def product(*args, repeat=1):
    pools = [tuple(pool) for pool in args] * repeat
    result = [[]]
    for pool in pools:
        result = [x + [y] for x in result for y in pool]
    for prod in result:
        yield tuple(prod)


def permutations(iterable, r=None):
    pool = tuple(iterable)
    n = len(pool)
    r = n if r is None else r
    if r > n:
        return
    indices = list(range(n))
    cycles = list(range(n, n - r, -1))
    yield tuple(pool[i] for i in indices[:r])
    while n:
        for i in reversed(range(r)):
            cycles[i] -= 1
            if cycles[i] == 0:
                indices[i:] = indices[i + 1 :] + indices[i : i + 1]
                cycles[i] = n - i
            else:
                j = cycles[i]
                indices[i], indices[-j] = indices[-j], indices[i]
                yield tuple(pool[i] for i in indices[:r])
                break
        else:
            return


def combinations(iterable, r):
    pool = tuple(iterable)
    n = len(pool)
    if r > n:
        return
    indices = list(range(r))
    yield tuple(pool[i] for i in indices)
    while True:
        for i in reversed(range(r)):
            if indices[i] != i + n - r:
                break
        else:
            return
        indices[i] += 1
        for j in range(i + 1, r):
            indices[j] = indices[j - 1] + 1
        yield tuple(pool[i] for i in indices)


def combinations_with_replacement(iterable, r):
    pool = tuple(iterable)
    n = len(pool)
    if not n and r:
        return
    indices = [0] * r
    yield tuple(pool[i] for i in indices)
    while True:
        for i in reversed(range(r)):
            if indices[i] != n - 1:
                break
        else:
            return
        indices[i:] = [indices[i] + 1] * (r - i)
        yield tuple(pool[i] for i in indices)


def pairwise(iterable):
    it = iter(iterable)
    try:
        a = next(it)
    except StopIteration:
        return
    for b in it:
        yield a, b
        a = b


def batched(iterable, n):
    if n < 1:
        raise ValueError("n must be at least one")
    it = iter(iterable)
    while True:
        batch = tuple(islice(it, n))
        if not batch:
            return
        yield batch
