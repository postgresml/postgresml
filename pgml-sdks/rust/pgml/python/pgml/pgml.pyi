
def py_init_logger(level: Optional[str] = "", format: Optional[str] = "") -> None

Json = Any
DateTime = int

# Top of file key: A12BECOD!
from typing import List, Dict, Optional, Self, Any


class Builtins:
	def __init__(self, database_url: Optional[str] = "Default set in Rust. Please check the documentation.") -> Self
		...
	def query(self, query: str) -> QueryRunner
		...
	async def transform(self, task: Json, inputs: List[str], args: Optional[Json] = Any) -> Json
		...

class Collection:
	def __init__(self, name: str, database_url: Optional[str] = "Default set in Rust. Please check the documentation.") -> Self
		...
	async def add_pipeline(self, pipeline: Pipeline) -> None
		...
	async def remove_pipeline(self, pipeline: Pipeline) -> None
		...
	async def enable_pipeline(self, pipeline: Pipeline) -> None
		...
	async def disable_pipeline(self, pipeline: Pipeline) -> None
		...
	async def upsert_documents(self, documents: List[Json], strict: Optional[bool] = True) -> None
		...
	async def get_documents(self, last_id: Optional[int] = 1, limit: Optional[int] = 1) -> List[Json]
		...
	async def vector_search(self, query: str, pipeline: Pipeline, query_parameters: Optional[Json] = Any, top_k: Optional[int] = 1) -> List[tuple[float, str, Json]]
		...
	async def archive(self) -> None
		...
	def query(self) -> QueryBuilder
		...
	async def get_pipelines(self) -> List[Pipeline]
		...
	async def get_pipeline(self, name: str) -> Pipeline
		...
	async def exists(self) -> bool
		...

class Model:
	def __init__(self, name: Optional[str] = "Default set in Rust. Please check the documentation.", source: Optional[str] = "Default set in Rust. Please check the documentation.", parameters: Optional[Json] = Any) -> Self
		...

class Pipeline:
	def __init__(self, name: str, model: Optional[Model] = Any, splitter: Optional[Splitter] = Any, parameters: Optional[Json] = Any) -> Self
		...
	async def get_status(self) -> PipelineSyncData
		...
	async def to_dict(self) -> Json
		...

class QueryBuilder:
	def limit(self, limit: int) -> Self
		...
	def filter(self, filter: Json) -> Self
		...
	def vector_recall(self, query: str, pipeline: Pipeline, query_parameters: Optional[Json] = Any) -> Self
		...
	async def fetch_all(self) -> List[tuple[float, str, Json]]
		...
	def to_full_string(self) -> str
		...

class QueryRunner:
	async def fetch_all(self) -> Json
		...
	async def execute(self) -> None
		...
	def bind_string(self, bind_value: str) -> Self
		...
	def bind_int(self, bind_value: int) -> Self
		...
	def bind_float(self, bind_value: float) -> Self
		...
	def bind_bool(self, bind_value: bool) -> Self
		...
	def bind_json(self, bind_value: Json) -> Self
		...

class Splitter:
	def __init__(self, name: Optional[str] = "Default set in Rust. Please check the documentation.", parameters: Optional[Json] = Any) -> Self
		...
