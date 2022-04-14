import unittest
import pgml

class TestRegression(unittest.TestCase):
    def test_init(self):
        pgml.model.train("Test", "regression", "test", "test_y")
        self.assertTrue(True)
