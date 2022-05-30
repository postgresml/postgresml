# stub out plpy
from . import plpy
import sys

sys.modules["plpy"] = plpy

import time
import unittest
from pgml_extension import model


class TestModel(unittest.TestCase):
    def test_the_world(self):
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "name": "Test",
                    "task": "regression",
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "relation_name": "test",
                    "y_column_name": "test_y",
                    "test_size": 0.25,
                    "test_sampling": "random",
                    "status": "new",
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        plpy.add_mock_result("OK")
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "relation_name": "test",
                    "y_column_name": "test_y",
                    "test_size": 0.25,
                    "test_sampling": "random",
                    "status": "created",
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "project_id": 1,
                    "snapshot_id": 1,
                    "algorithm_name": "linear",
                    "status": "new",
                    "r2_score": None,
                    "mean_squared_error": None,
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        plpy.add_mock_result(
            [
                {"a": 1, "b": 2, "test_y": 3},
                {"a": 2, "b": 3, "test_y": 4},
                {"a": 3, "b": 4, "test_y": 5},
            ]
        )
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "project_id": 1,
                    "snapshot_id": 1,
                    "algorithm_name": "linear",
                    "status": "new",
                    "r2_score": None,
                    "mean_squared_error": None,
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )

        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "project_id": 1,
                    "snapshot_id": 1,
                    "algorithm_name": "linear",
                    "status": "new",
                    "r2_score": None,
                    "mean_squared_error": None,
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        plpy.add_mock_result(
            [
                {"a": 1, "b": 2, "test_y": 3},
                {"a": 2, "b": 3, "test_y": 4},
                {"a": 3, "b": 4, "test_y": 5},
            ]
        )
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "project_id": 1,
                    "snapshot_id": 1,
                    "algorithm_name": "linear",
                    "status": "new",
                    "r2_score": None,
                    "mean_squared_error": None,
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )

        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "project_id": 1,
                    "snapshot_id": 1,
                    "algorithm_name": "linear",
                    "status": "new",
                    "r2_score": None,
                    "mean_squared_error": None,
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        plpy.add_mock_result(
            [
                {"a": 1, "b": 2, "test_y": 3},
                {"a": 2, "b": 3, "test_y": 4},
                {"a": 3, "b": 4, "test_y": 5},
            ]
        )
        plpy.add_mock_result(
            [
                {
                    "id": 1,
                    "project_id": 1,
                    "snapshot_id": 1,
                    "algorithm_name": "linear",
                    "status": "new",
                    "r2_score": None,
                    "mean_squared_error": None,
                    "created_at": time.time(),
                    "updated_at": time.time(),
                }
            ]
        )
        model.train("Test", "regression", "test", "test_y")
