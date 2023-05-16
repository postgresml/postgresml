import unittest
from pgml import Database, Collection
from pgml.dbutils import *
import hashlib

class TestCollectionEmbedding(unittest.TestCase):

    def setUp(self) -> None:
        conninfo = "postgres://postgres@127.0.0.1:5433/pgml_development"
        self.db = Database(conninfo)
        self.collection_name = "test_collection_1"
        self.documents = [
            {
            "id": hashlib.md5(f"abcded-{i}".encode('utf-8')).hexdigest(),
            "text":f"Lorem ipsum {i}",
            "metadata": {"source": "test_suite"}
            }
            for i in range(4, 7)
        ]
        self.documents_no_ids = [
            {
            "text":f"Lorem ipsum {i}",
            "metadata": {"source": "test_suite_no_ids"}
            }
            for i in range(1, 4)
        ]
        
        self.collection = self.db.create_collection(self.collection_name)
    
    def test_create_collection(self):
        assert isinstance(self.collection,Collection)
    
    def test_upsert_documents(self):
        self.collection.upsert_documents(self.documents)
        conn = self.db.pool.getconn()
        results = run_select_statement(conn,"SELECT id FROM %s"%self.collection.documents_table)
        self.db.pool.putconn(conn)
        assert len(results) >= len(self.documents)
    
    def test_upsert_documents_no_ids(self):
        self.collection.upsert_documents(self.documents_no_ids)
        conn = self.db.pool.getconn()
        results = run_select_statement(conn,"SELECT id FROM %s"%self.collection.documents_table)
        self.db.pool.putconn(conn)
        assert len(results) >= len(self.documents_no_ids)
    
    
    # def tearDown(self) -> None:
    #     self.db.delete_collection(self.collection_name)


    
    
