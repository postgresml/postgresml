from plpy import quote_literal
import json
import re
from .exceptions import PgMLException
import numpy


def q(obj):
    if type(obj) == str:
        return quote_literal(obj)
    elif type(obj) == dict:
        return quote_literal(json.dumps(obj))
    elif obj is None:
        return "NULL"
    elif type(obj) in [bool, numpy.bool_]:
        if obj:
            return "TRUE"
        else:
            return "FALSE"
    elif type(obj) in [int, float, numpy.int64, numpy.float64]:
        return str(obj)

    raise PgMLException(f"Unhandled postgres type: {type(obj)}")


def c(column_name):
    if column_name[0] == '"' and column_name[-1] == '"':
        return column_name
    return '"' + column_name + '"'
