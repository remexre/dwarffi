import ctypes
import sys

lookup = {k: v for [k, v] in ffi_values}


def Module():
    # Louis: how do I actually do this
    d = {}

    class Module():
        def __contains__(self, name):
            return name in d

        def __dir__(self):
            return list(d.keys())

        def __getattr__(self, name):
            return d[name]

        def __setattr__(self, name, value):
            d[name] = value

        def __getitem__(self, name):
            return d[name]

        def __setitem__(self, name, value):
            d[name] = value

    return Module()


values = Module()

for [_, value] in ffi_values:
    cell = [values]
    for m in value['module']:
        if m not in cell[0]:
            cell[0][m] = Module()
        cell = [cell[0][m]]
    name = value['name']
    if name in cell[0]:
        print(cell[0][name])
        print(value)
        raise Exception('duplicate name {}'.format(repr(name)))
    cell[0][name] = value


class Root:
    def __getattr__(self, name):
        if name == '_ffi_values':
            return ffi_values
        elif name in values:
            return values[name]
        raise AttributeError(
            'module {} has no attribute {}'.format(
                repr(__name__), repr(name)))


sys.modules[__name__] = Root()
