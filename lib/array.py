"""array.array backed by a list."""

typecodes = "bBuhHiIlLqQfd"
_INT = "bBhHiIlLqQ"
_SIZE = {"b": 1, "B": 1, "u": 4, "h": 2, "H": 2, "i": 4, "I": 4, "l": 8, "L": 8, "q": 8, "Q": 8, "f": 4, "d": 8}


class array:
    def __init__(self, typecode, initializer=()):
        if typecode not in _SIZE:
            raise ValueError("bad typecode (must be b, B, u, h, H, i, I, l, L, q, Q, f or d)")
        self.typecode = typecode
        self.itemsize = _SIZE[typecode]
        self._a = [self._check(v) for v in initializer]

    def _check(self, v):
        if self.typecode in _INT:
            if not isinstance(v, int):
                raise TypeError("an integer is required")
            return v
        if self.typecode == "u":
            return v
        return float(v)

    def __len__(self):
        return len(self._a)

    def __getitem__(self, i):
        if isinstance(i, slice):
            r = array(self.typecode)
            r._a = self._a[i]
            return r
        return self._a[i]

    def __setitem__(self, i, v):
        if isinstance(i, slice):
            self._a[i] = v._a if isinstance(v, array) else v
        else:
            self._a[i] = self._check(v)

    def __delitem__(self, i):
        del self._a[i]

    def __iter__(self):
        return iter(self._a)

    def __contains__(self, v):
        return v in self._a

    def __eq__(self, other):
        return isinstance(other, array) and self._a == other._a

    def __add__(self, other):
        if not isinstance(other, array) or other.typecode != self.typecode:
            return NotImplemented
        r = array(self.typecode)
        r._a = self._a + other._a
        return r

    def __iadd__(self, other):
        self.extend(other)
        return self

    def __mul__(self, n):
        r = array(self.typecode)
        r._a = self._a * n
        return r

    __rmul__ = __mul__

    def __repr__(self):
        if not self._a:
            return f"array('{self.typecode}')"
        return f"array('{self.typecode}', {self._a!r})"

    def append(self, v):
        self._a.append(self._check(v))

    def extend(self, it):
        for v in it:
            self.append(v)

    def insert(self, i, v):
        self._a.insert(i, self._check(v))

    def pop(self, i=-1):
        return self._a.pop(i)

    def remove(self, v):
        self._a.remove(v)

    def index(self, v):
        return self._a.index(v)

    def count(self, v):
        return self._a.count(v)

    def reverse(self):
        self._a.reverse()

    def tolist(self):
        return list(self._a)

    def buffer_info(self):
        return (id(self), len(self._a))


ArrayType = array
