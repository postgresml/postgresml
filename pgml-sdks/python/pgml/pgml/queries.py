from typing import Any
from pypika.functions import Function, Cast
from typing import Dict, List, Optional
from pypika import JSON, Array, Table, AliasedQuery, Order
from pypika import Query as PyPikaQuery
import json
from rich import print as rprint


class Embed(Function):
    def __init__(
        self,
        transformer: str,
        text: str,
        parameters: Dict[str, Any] = {},
        alias: str = "embedding",
    ) -> None:
        super(Embed, self).__init__(
            "pgml.embed", transformer, text, JSON(parameters), alias=alias
        )


class CosineDistance(Function):
    def __init__(self, lhs: Array, rhs: Array, alias: str = "cosine") -> None:
        super(CosineDistance, self).__init__("cosine_distance", lhs, rhs, alias=alias)


class Transform(Function):
    def __init__(
        self,
        task: str | Dict[str, Any],
        inputs: List,
        args: Dict[str, Any] = {},
        alias: str = "transform",
    ) -> None:
        super(Transform, self).__init__(
            "pgml.transform", task=task, inputs=inputs, args=args, alias=alias
        )
        self.task = task
        self.inputs = inputs
        self.args = args

    def get_function_sql(self, **kwargs: Any) -> str:
        if isinstance(self.task, str):
            return "pgml.transform(task => '{}', inputs => ARRAY{}, args => '{}'::JSONB)".format(
                self.task, self.inputs, self.args
            )
        elif isinstance(self.task, dict):
            return "pgml.transform(task => '{}'::JSONB, inputs => ARRAY{}, args => '{}'::JSONB)".format(
                json.dumps(self.task), self.inputs, self.args
            )
