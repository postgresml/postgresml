from plpy import quote_literal


def q(obj):
    if type(obj) == str:
        return quote_literal(obj)
    return obj
