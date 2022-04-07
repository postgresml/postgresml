from pgml.validate import check_type
from pgml.exceptions import PgMLException

import pytest


def test_check_type():
    row = {
        "col1": 1,
        "col2": "text",
        "col3": 1.5,
    }

    check_type(row)

    row = {
        "col1": 1,
        "col2": Exception(),
    }

    with pytest.raises(PgMLException):
        check_type(row)
