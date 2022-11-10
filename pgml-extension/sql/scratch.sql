CREATE TABLE test (
    name TEXT,
    id INT4,
    description varchar(10),
    big INT8,
    value FLOAT4,
    category BOOL,
    image FLOAT4[],
    target FLOAT4
);

insert into test VALUES
('one', 2, 'hi', 1, 1.0, true, ARRAY[1, 1, 1, 1], 1),
('one', 4, 'hi', 2, 2.0, false, ARRAY[2, 2, 2, 2], 2),
('one', 6, 'hi', 3, 3.0, true, ARRAY[3, 3, 3, 3], 3),
('one', 8, 'hi', 4, 4.0, false, ARRAY[4, 4, 4, 4], 4),
('two', 10, 'hi', 5, 5.0, true, ARRAY[5, 5, 5, 5], 5),
('two', 12, 'hi', 6, 6.0, false, ARRAY[6, 6, 6, 6], 6),
('two', 14, 'hi', 7, 7.0, true, ARRAY[7, 7, 7, 7], 7),
('two', 16, 'hi', 8, 8.0, false, ARRAY[8, 8, 8, 8], 8);

select pgml.train('test', 'regression', 'test', 'target');
select pgml.predict('test', ('one', 2, 'hi', 1, 1.0, true, ARRAY[1, 1, 1, 1]));
