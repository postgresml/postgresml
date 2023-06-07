from typing import Any
from pypika.functions import Function
from typing import Dict
from pypika import JSON, Array

class Embed(Function):
    def __init__(self, transformer: str, text: str, parameters: Dict[str,Any] = {}, alias: str = "embedding") -> None:
        super(Embed,self).__init__('pgml.embed', transformer, text, JSON(parameters), alias=alias)

class CosineDistance(Function):
    def __init__(self, lhs: Array, rhs: Array, alias: str = "cosine") -> None:
        super(CosineDistance,self).__init__('cosine_distance', lhs, rhs, alias=alias)