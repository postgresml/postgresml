import sys
import numpy
numpy.set_printoptions(threshold=sys.maxsize)

embeddings = numpy.random.rand(10_000, 128)
print(embeddings)
