#include <stdio.h>

#include "pgml.h"

int main() {
  // Create the Collection and Pipeline
  CollectionC * collection = CollectionC_new("test_c", NULL);
  PipelineC * pipeline = PipelineC_new("test_c", "{\"text\": {\"splitter\": {\"model\": \"recursive_character\"},\"semantic_search\": {\"model\": \"intfloat/e5-small\"}}}");

  // Add the Pipeline to the Collection
  CollectionC_add_pipeline(collection, pipeline);

  // Upsert the documents
  char * documents_to_upsert[2] = {"{\"id\": \"doc1\", \"text\": \"test1\"}", "{\"id\": \"doc2\", \"text\": \"test2\"}"};
  CollectionC_upsert_documents(collection, documents_to_upsert, 2, NULL);

  // Retrieve the documents
  unsigned long r_size = 0;
  char** documents = CollectionC_get_documents(collection, NULL, &r_size);

  // Print the documents
  printf("\n\nPrinting documents:\n");
  int i;
  for (i = 0; i < r_size; i++) {
    printf("Document %u -> %s\n", i, documents[i]);
  }

  // Search over the documents
  r_size = 0;
  char** results = CollectionC_vector_search(collection, "{\"query\": {\"fields\": {\"text\": {\"query\": \"Test query!\"}}}, \"limit\": 5}", pipeline, &r_size);
  printf("\n\nPrinting results:\n");
  for (i = 0; i < r_size; i++) {
    printf("Result %u -> %s\n", i, results[i]);
  }

  return 0;
}
