"""Abstract base classes for containers (structural isinstance checks)."""


def _has(cls, *methods):
    return all(any(m in vars(c) for c in cls.__mro__) for m in methods)


class _ABC:
    _methods = ()

    @classmethod
    def __subclasshook__(cls, C):
        return _has(C, *cls._methods)


class Hashable(_ABC):
    _methods = ("__hash__",)


class Iterable(_ABC):
    _methods = ("__iter__",)


class Iterator(Iterable):
    _methods = ("__iter__", "__next__")


class Reversible(Iterable):
    _methods = ("__reversed__", "__iter__")


class Generator(Iterator):
    _methods = ("__iter__", "__next__", "send", "throw", "close")


class Sized(_ABC):
    _methods = ("__len__",)


class Container(_ABC):
    _methods = ("__contains__",)


class Callable(_ABC):
    _methods = ("__call__",)


class Collection(Sized, Iterable, Container):
    _methods = ("__len__", "__iter__", "__contains__")


class Sequence(Reversible, Collection):
    _methods = ("__getitem__", "__len__")


class MutableSequence(Sequence):
    _methods = ("__getitem__", "__setitem__", "__delitem__", "__len__", "insert")


class Mapping(Collection):
    _methods = ("__getitem__", "__len__", "__iter__")


class MutableMapping(Mapping):
    _methods = ("__getitem__", "__setitem__", "__delitem__", "__len__", "__iter__")


class Set(Collection):
    _methods = ("__len__", "__iter__", "__contains__")


class MutableSet(Set):
    _methods = ("__len__", "__iter__", "__contains__", "add", "discard")
