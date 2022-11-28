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
('one', 2, NULL, 1, 1.0, true, ARRAY[1, 1, 1, 1], 1),
('one', 4, 'bye', 2, 2.0, NULL, ARRAY[2, 2, 2, 2], 2),
('one', 6, 'hi', 3, NULL, true, ARRAY[3, 3, 3, 3], 3),
('one', NULL, 'hi', 4, 4.0, false, ARRAY[4, 4, 4, 4], 4),
(NULL, 10, 'hi', 5, 5.0, true, ARRAY[5, 5, 5, 5], 5),
('two', 12, 'hi', 6, 6.0, false, ARRAY[6, 6, 6, 6], 6),
('two', 14, 'what', 7, 7.0, true, ARRAY[7, 7, 7, 7], 7),
('two', 16, 'hi', 8, 8.0, false, ARRAY[8, 8, 8, 8], 8);

select pgml.train('test', 'regression', 'test', 'target', preprocess => '{
    "name": {"impute": "mode", "encode": {"ordinal": ["one"]}}
    }'
);
select pgml.predict('test', ('one', 2, 'hi', 1, 1.0, true, ARRAY[1, 1, 1, 1]));

preprocess => {
“TEXT” => [
	 	{
            encode: “target” |
                | {“one_hot”: {limit: 0, min_frequency: 0.01}, -- default to N - 1
                | {“ordinal”: [‘a’, ‘b’, ‘c’]},
            impute: “mean” | “median” | “mode” | “min” | “max” | “missing” | “error” | CONSTANT
            scale: [
                “standard”, -- zero mean, unit variance
                “min_max”, -- zero min, one max
                “max_abs”, -- not necessary since our data is not sparse
                “robust”, -- remove outliers
                “quantile”, -- quantile non linear scaling
                “box_cox”, -- box_cox non linear scaling
                “yeo_johnson” -- yeo_johnson non linear scaling
                “none”,
            ]
        }
],
“INT” => [
encode, scale, impute
]
“FLOAT” => {
	scale, impute
]
“DATE” => [
	seasonal, scale, impute
]
“TIME” => [
	scale, impute
]
TIMESTAMP => [
Date: seasonal, scale, impute
Time: scale, impute
]

}
