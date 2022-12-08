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
[0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 4.0, 6.0, 6.0,
10.0, 12.0, 3.0, 2.0, 3.0, 3.0, 3.0, 3.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, -1.401826, -0.86266214, 0.2156656, 0.2156656, 0.75482947, 1.2939934, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, -1.46385, -0.87831, -0.29277, 0.29277, 0.87831, 1.46385, -1.46385, -0.87831, -0.29277, 0.29277, 0.87831, 1.46385, -1.46385, -0.87831, -0.29277, 0.29277, 0.87831, 1.46385, -1.46385, -0.87831, -0.29277, 0.29277, 0.87831, 1.46385
('one', 4, 'bye', 2, 2.0, NULL, ARRAY[2, 2, 2, 2], 2),
('one', 6, 'hi', 3, NULL, true, ARRAY[3, 3, 3, 3], 3),
('one', NULL, 'hi', 4, 4.0, false, ARRAY[4, 4, 4, 4], 4),
(NULL, 10, 'hi', 5, 5.0, true, ARRAY[5, 5, 5, 5], 5),
('two', 12, 'hi', 6, 6.0, false, ARRAY[6, 6, 6, 6], 6),
('two', 14, 'what', 7, 7.0, true, ARRAY[7, 7, 7, 7], 7),
('two', 16, 'hi', 8, 8.0, false, ARRAY[8, 8, 8, 8], 8);

select pgml.train('test', 'regression', 'test', 'target', preprocess => '{
    "name": {"impute": "mode", "encode": {"ordinal": ["one", "two"]}}
    }'
);
select pgml.deploy('test', 'most_recent');
select pgml.predict('test', ('one', 2, 'hi', 1, 1.0, true, ARRAY[1, 1, 1, 1]));

select pgml.train('test', 'regression', 'test', 'target', preprocess => '{
    "name": {"scale": "standard" },
    "id": {"scale": "standard" },
    "description": {"scale": "standard" },
    "big": {"scale": "min_max" },
    "value": {"scale": "preserve" },
    "category": {"scale": "robust" },
    "image": {"scale": "max_abs" }
    }'
);

select pgml.train('diabetes', 'regression', 'pgml.diabetes', 'target', algorithm => 'linear', preprocess => '{
    "age": {"scale": "preserve" },
    "sex": {"scale": "preserve" },
    "bmi": {"scale": "preserve" },
    "bp": {"scale": "preserve" },
    "s1": {"scale": "preserve" },
    "s2": {"scale": "preserve" },
    "s3": {"scale": "preserve" },
    "s4": {"scale": "preserve" },
    "s5": {"scale": "preserve" },
    "s6": {"scale": "preserve" }
    }'
);

select pgml.train('diabetes', 'regression', 'pgml.diabetes', 'target', algorithm => 'lasso', preprocess => '{
    "age": {"scale": "standard" },
    "sex": {"scale": "standard" },
    "bmi": {"scale": "standard" },
    "bp": {"scale": "standard" },
    "s1": {"scale": "standard" },
    "s2": {"scale": "standard" },
    "s3": {"scale": "standard" },
    "s4": {"scale": "standard" },
    "s5": {"scale": "standard" },
    "s6": {"scale": "standard" }
    }'
);

select target, pgml.predict('diabetes', (age, sex, bmi, bp, s1, s2, s3, s4, s5, s6))
FROM pgml.diabetes;

INFO:  json: JsonB(Array [Object {"name": String("name"), "size": Number(1), "label": Bool(false), "pg_type": String("text"), "nullable": Bool(true), "position": Number(1), "statistics": Object {"max": Number(1.0), "min": Number(0.0), "mean": Number(0.20000000298023224), "mode": Number(0.0), "median": Number(0.0), "max_abs": Number(1.0), "missing": Number(1), "std_dev": Number(0.4000000059604645), "distinct": Number(2), "variance": Number(0.1600000113248825), "ventiles": Array [Number(0.0), Null, Null, Null, Number(0.0), Null, Null, Null, Number(0.0), Null, Null, Null, Number(0.0), Null, Null, Null, Number(1.0), Null, Null], "histogram": Array [Number(4), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(1)], "categories": Object {"one": Object {"value": Number(0.0), "members": Number(4)}, "two": Object {"value": Number(1.0), "members": Number(3)}, "NULL": Object {"value": Null, "members": Number(1)}}}, "preprocessor": Object {"scale": String("standard"), "encode": Object {"ordinal": Array [String("one"), String("two")]}, "impute": String("mode")}}, Object {"name": String("id"), "size": Number(1), "label": Bool(false), "pg_type": String("int4"), "nullable": Bool(true), "position": Number(2), "statistics": Object {"max": Number(12.0), "min": Number(2.0), "mean": Number(6.800000190734863), "mode": Number(6.0), "median": Number(6.0), "max_abs": Number(12.0), "missing": Number(1), "std_dev": Number(3.7094473838806152), "distinct": Number(5), "variance": Number(13.75999927520752), "ventiles": Array [Number(2.0), Null, Null, Null, Number(4.0), Null, Null, Null, Number(6.0), Null, Null, Null, Number(10.0), Null, Null, Null, Number(12.0), Null, Null], "histogram": Array [Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(1)], "categories": Object {}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}, Object {"name": String("description"), "size": Number(1), "label": Bool(false), "pg_type": String("varchar"), "nullable": Bool(true), "position": Number(3), "statistics": Object {"max": Number(3.0), "min": Number(2.0), "mean": Number(2.799999952316284), "mode": Number(3.0), "median": Number(3.0), "max_abs": Number(3.0), "missing": Number(1), "std_dev": Number(0.4000000059604645), "distinct": Number(2), "variance": Number(0.1599999964237213), "ventiles": Array [Number(2.0), Null, Null, Null, Number(3.0), Null, Null, Null, Number(3.0), Null, Null, Null, Number(3.0), Null, Null, Null, Number(3.0), Null, Null], "histogram": Array [Number(1), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(4)], "categories": Object {"hi": Object {"value": Number(3.0), "members": Number(5)}, "bye": Object {"value": Number(2.0), "members": Number(1)}, "NULL": Object {"value": Null, "members": Number(1)}, "what": Object {"value": Number(4.0), "members": Number(1)}}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}, Object {"name": String("big"), "size": Number(1), "label": Bool(false), "pg_type": String("int8"), "nullable": Bool(true), "position": Number(4), "statistics": Object {"max": Number(6.0), "min": Number(1.0), "mean": Number(3.5), "mode": Number(3.0), "median": Number(4.0), "max_abs": Number(6.0), "missing": Number(0), "std_dev": Number(1.7078251838684082), "distinct": Number(6), "variance": Number(2.9166667461395264), "ventiles": Array [Number(1.0), Null, Null, Number(2.0), Null, Null, Number(3.0), Null, Null, Null, Number(4.0), Null, Null, Number(5.0), Null, Null, Number(6.0), Null, Null], "histogram": Array [Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(1)], "categories": Object {}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}, Object {"name": String("value"), "size": Number(1), "label": Bool(false), "pg_type": String("float4"), "nullable": Bool(true), "position": Number(5), "statistics": Object {"max": Number(6.0), "min": Number(1.0), "mean": Number(3.5999999046325684), "mode": Number(4.0), "median": Number(4.0), "max_abs": Number(6.0), "missing": Number(1), "std_dev": Number(1.8547236919403076), "distinct": Number(5), "variance": Number(3.43999981880188), "ventiles": Array [Number(1.0), Null, Null, Null, Number(2.0), Null, Null, Null, Number(4.0), Null, Null, Null, Number(5.0), Null, Null, Null, Number(6.0), Null, Null], "histogram": Array [Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(1)], "categories": Object {}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}, Object {"name": String("category"), "size": Number(1), "label": Bool(false), "pg_type": String("bool"), "nullable": Bool(true), "position": Number(6), "statistics": Object {"max": Number(1.0), "min": Number(0.0), "mean": Number(0.6000000238418579), "mode": Number(1.0), "median": Number(1.0), "max_abs": Number(1.0), "missing": Number(1), "std_dev": Number(0.4898979365825653), "distinct": Number(2), "variance": Number(0.2399999797344208), "ventiles": Array [Number(0.0), Null, Null, Null, Number(0.0), Null, Null, Null, Number(1.0), Null, Null, Null, Number(1.0), Null, Null, Null, Number(1.0), Null, Null], "histogram": Array [Number(2), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(0), Number(3)], "categories": Object {}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}, Object {"name": String("image"), "size": Number(4), "label": Bool(false), "pg_type": String("float4[]"), "nullable": Bool(true), "position": Number(7), "statistics": Object {"max": Number(6.0), "min": Number(1.0), "mean": Number(3.5), "mode": Number(3.0), "median": Number(4.0), "max_abs": Number(6.0), "missing": Number(0), "std_dev": Number(1.7078251838684082), "distinct": Number(24), "variance": Number(2.9166667461395264), "ventiles": Array [Number(1.0), Null, Null, Number(2.0), Null, Null, Number(3.0), Null, Null, Null, Number(4.0), Null, Null, Number(5.0), Null, Null, Number(6.0), Null, Null], "histogram": Array [Number(4), Number(0), Number(0), Number(0), Number(4), Number(0), Number(0), Number(0), Number(4), Number(0), Number(0), Number(0), Number(4), Number(0), Number(0), Number(0), Number(4), Number(0), Number(0), Number(4)], "categories": Object {}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}, Object {"name": String("target"), "size": Number(1), "label": Bool(true), "pg_type": String("float4"), "nullable": Bool(true), "position": Number(8), "statistics": Object {"max": Number(6.0), "min": Number(1.0), "mean": Number(3.5), "mode": Number(3.0), "median": Number(4.0), "max_abs": Number(6.0), "missing": Number(0), "std_dev": Number(1.7078251838684082), "distinct": Number(6), "variance": Number(2.9166667461395264), "ventiles": Array [Number(1.0), Null, Null, Number(2.0), Null, Null, Number(3.0), Null, Null, Null, Number(4.0), Null, Null, Number(5.0), Null, Null, Number(6.0), Null, Null], "histogram": Array [Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(0), Number(1), Number(0), Number(0), Number(1)], "categories": Object {}}, "preprocessor": Object {"scale": String("standard"), "encode": String("label"), "impute": String("mode")}}])
ERROR:  called `Result::unwrap()` on an `Err` value: Error("invalid type: null, expected f32", line: 0, column: 0)

select pgml.train('diabetes', 'regression', 'pgml.diabetes', 'target', algorithm => 'lasso', preprocess => '{
    "age": {"scale": "min_max" },
    "sex": {"scale": "min_max" },
    "bmi": {"scale": "min_max" },
    "bp": {"scale": "min_max" },
    "s1": {"scale": "min_max" },
    "s2": {"scale": "min_max" },
    "s3": {"scale": "min_max" },
    "s4": {"scale": "min_max" },
    "s5": {"scale": "min_max" },
    "s6": {"scale": "min_max" }
    }'
);


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
