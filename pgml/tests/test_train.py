from pgml.train import train


class PlPyIterator:
    def __init__(self, values):
        self._values = values
        self._returned = False

    def fetch(self, n):
        if self._returned:
            return
        else:
            self._returned = True
            return self._values


def test_train():
    it = PlPyIterator(
        [
            {
                "value": 5,
                "weight": 5,
            },
            {
                "value": 34,
                "weight": 5,
            },
        ]
    )

    train(it, y_column="weight", name="test", save=False)
