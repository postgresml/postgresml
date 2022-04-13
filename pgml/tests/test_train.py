import unittest
import pgml

class TestRegression(unittest.TestCase):
    def test_init(self):
        pgml.model.Regression("Test", "test", "test_y")
        self.assertTrue(True)
