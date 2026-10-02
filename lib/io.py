"""In-memory text streams."""


class StringIO:
    def __init__(self, initial_value="", newline="\n"):
        self._parts = [initial_value] if initial_value else []
        self._pos = 0
        self.closed = False

    def _value(self):
        if len(self._parts) > 1:
            self._parts = ["".join(self._parts)]
        return self._parts[0] if self._parts else ""

    def write(self, s):
        if not isinstance(s, str):
            raise TypeError(f"string argument expected, got '{type(s).__name__}'")
        v = self._value()
        if self._pos == len(v):
            self._parts.append(s)
        else:
            self._parts = [v[: self._pos] + s + v[self._pos + len(s) :]]
        self._pos += len(s)
        return len(s)

    def writelines(self, lines):
        for line in lines:
            self.write(line)

    def getvalue(self):
        return self._value()

    def read(self, size=-1):
        v = self._value()
        end = len(v) if size is None or size < 0 else self._pos + size
        s = v[self._pos : end]
        self._pos += len(s)
        return s

    def readline(self, size=-1):
        v = self._value()
        i = v.find("\n", self._pos)
        end = len(v) if i < 0 else i + 1
        if size is not None and size >= 0:
            end = min(end, self._pos + size)
        s = v[self._pos : end]
        self._pos = end
        return s

    def readlines(self):
        out = []
        while True:
            line = self.readline()
            if not line:
                return out
            out.append(line)

    def __iter__(self):
        return iter(self.readlines())

    def seek(self, pos, whence=0):
        self._pos = pos if whence == 0 else (self._pos + pos if whence == 1 else len(self._value()) + pos)
        return self._pos

    def tell(self):
        return self._pos

    def truncate(self, size=None):
        v = self._value()
        size = self._pos if size is None else size
        self._parts = [v[:size]]
        return size

    def close(self):
        self.closed = True

    def flush(self):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()
