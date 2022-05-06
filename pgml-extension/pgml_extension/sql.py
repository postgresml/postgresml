from plpy import quote_literal
import json
import re
from pgml_extension.exceptions import PgMLException
import numpy

def q(obj):
    if type(obj) == str:
        return quote_literal(obj)
    elif type(obj) == dict:
        return quote_literal(json.dumps(obj))
    elif obj is None:
        return "NULL"
    elif type(obj) in [int, float, numpy.int64, numpy.float64]:
        return obj

    raise PgMLException(f"Unhandled postgres type: {type(obj)}")
