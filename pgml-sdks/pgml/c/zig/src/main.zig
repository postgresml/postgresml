const pgml = @cImport({
    // See https://github.com/ziglang/zig/issues/515
    // @cDefine("_NO_CRT_STDIO_INLINE", "1");
    // @cInclude("./../pgml.h");
    @cInclude("./../pgml.h");
});

pub fn main() void {
    // Create the Collection and Pipeline
    var collection: *pgml.CollectionC = pgml.CollectionC_new(@constCast("test_c"), null);
    var pipeline: *pgml.PipelineC = pgml.PipelineC_new(@constCast("test_c"), @constCast("{\"text\": {\"splitter\": {\"model\": \"recursive_character\"},\"semantic_search\": {\"model\": \"intfloat/e5-small\"}}}"));

    // Add the Pipeline to the Collection
    pgml.CollectionC_add_pipeline(collection, pipeline);

    // Upsert the documents
    // const documents_to_upsert: [2][]const u8 = .{ "{\"id\": \"doc1\", \"text\": \"test1\"}", "{\"id\": \"doc2\", \"text\": \"test2\"}" };
    // const c_documents_to_upsert: [*c][*c]pgml.JsonC = @as([*c][*c]pgml.JsonC, @ptrCast(@constCast(documents_to_upsert[0..2].ptr)));
    // pgml.CollectionC_upsert_documents(collection, c_documents_to_upsert, 2, null);
}

// test "simple test" {
//     // Create the Collection and Pipeline
//     var collection: *pgml.CollectionC = pgml.CollectionC_new(@constCast("test_c"), null);
//     var pipeline: *pgml.PipelineC = pgml.PipelineC_new(@constCast("test_c"), @constCast("{\"text\": {\"splitter\": {\"model\": \"recursive_character\"},\"semantic_search\": {\"model\": \"intfloat/e5-small\"}}}"));

//     // Add the Pipeline to the Collection
//     pgml.CollectionC_add_pipeline(collection, pipeline);

//     // Upsert the documents
//     // char * documents_to_upsert[2] = {"{\"id\": \"doc1\", \"text\": \"test1\"}", "{\"id\": \"doc2\", \"text\": \"test2\"}"};
//     // CollectionC_upsert_documents(collection, documents_to_upsert, 2, NULL);

//     // // Retrieve the documents
//     // unsigned long r_size = 0;
//     // char** documents = CollectionC_get_documents(collection, NULL, &r_size);
// }
