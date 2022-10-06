use pgx::*;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Dataset {
    pub x_train: Vec<f32>,
    pub y_train: Vec<f32>,
    pub x_test: Vec<f32>,
    pub y_test: Vec<f32>,
    pub num_features: usize,
    pub num_labels: usize,
    pub num_rows: usize,
    pub num_train_rows: usize,
    pub num_test_rows: usize,
    pub num_distinct_labels: usize,
}

impl Display for Dataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Dataset {{ num_features: {}, num_labels: {}, num_rows: {}, num_train_rows: {}, num_test_rows: {} }}",
            self.num_features, self.num_labels, self.num_rows, self.num_train_rows, self.num_test_rows,
        )
    }
}

impl Dataset {
    pub fn fold(&self, k: usize, folds: usize) -> Dataset {
        // TODO return an error here instead of copy
        if folds == 1 {
            return Dataset {
                x_train: self.x_train.clone(),
                y_train: self.y_train.clone(),
                x_test: self.x_test.clone(),
                y_test: self.y_test.clone(),
                num_features: self.num_features,
                num_labels: self.num_labels,
                num_rows: self.num_rows,
                num_train_rows: self.num_train_rows,
                num_test_rows: self.num_test_rows,
                num_distinct_labels: self.num_distinct_labels,
            };
        }
        let fold_test_size = self.num_train_rows / folds;
        let test_start = k * fold_test_size;
        let test_end = test_start + fold_test_size;
        let num_train_rows = self.num_train_rows - fold_test_size;

        let x_test_start = test_start * self.num_features;
        let x_test_end = test_end * self.num_features;
        let y_test_start = test_start * self.num_labels;
        let y_test_end = test_end * self.num_labels;

        let mut x_train = Vec::with_capacity(num_train_rows * self.num_features);
        x_train.extend_from_slice(&self.x_train[..x_test_start]);
        x_train.extend_from_slice(&self.x_train[x_test_end..]);
        let mut y_train = Vec::with_capacity(num_train_rows * self.num_labels);
        y_train.extend_from_slice(&self.y_train[..y_test_start]);
        y_train.extend_from_slice(&self.y_train[y_test_end..]);

        let x_test = self.x_train[x_test_start..x_test_end].to_vec();
        let y_test = self.y_train[y_test_start..y_test_end].to_vec();

        info!(
            "x_train: {} {}",
            self.num_train_rows * self.num_features,
            x_train.len()
        );
        info!(
            "x_test: {} {} {} {}",
            self.num_test_rows * self.num_features,
            x_test.len(),
            x_test_start,
            x_test_end
        );
        info!(
            "y_train: {} {}",
            self.num_train_rows * self.num_labels,
            y_train.len()
        );
        info!(
            "y_test: {} {} {} {}",
            self.num_test_rows * self.num_labels,
            y_test.len(),
            y_test_start,
            y_test_end
        );

        Dataset {
            x_train,
            y_train,
            x_test,
            y_test,
            num_features: self.num_features,
            num_labels: self.num_labels,
            num_rows: self.num_train_rows,
            num_train_rows,
            num_test_rows: fold_test_size,
            num_distinct_labels: self.num_distinct_labels,
        }
    }
}

fn run_with_args(query: &str, args: Vec<(PgOid, Option<pg_sys::Datum>)>) {
    Spi::execute(|mut client| {
        client.update(query, None, Some(args));
    })
}

pub fn load_diabetes(limit: Option<usize>) -> (String, i64) {
    let diabetes = smartcore::dataset::diabetes::load_dataset();
    Spi::run("DROP TABLE IF EXISTS pgml.diabetes");
    Spi::run(
        "CREATE TABLE pgml.diabetes (
        age FLOAT4, 
        sex FLOAT4, 
        bmi FLOAT4, 
        bp FLOAT4, 
        s1 FLOAT4, 
        s2 FLOAT4, 
        s3 FLOAT4, 
        s4 FLOAT4, 
        s5 FLOAT4, 
        s6 FLOAT4, 
        target INTEGER
    )",
    );
    // TODO replace run_with_args with upstream when PR is accepted
    run_with_args(
        "COMMENT ON TABLE pgml.diabetes IS '{description}'",
        vec![(
            PgBuiltInOids::TEXTOID.oid(),
            diabetes.description.into_datum(),
        )],
    );

    info!(
        "num_features: {}, num_samples: {}, feature_names: {:?}",
        diabetes.num_features, diabetes.num_samples, diabetes.feature_names
    );
    let limit = match limit {
        Some(limit) => {
            if limit > diabetes.num_samples {
                diabetes.num_samples
            } else {
                limit
            }
        }
        None => diabetes.num_samples,
    };
    for i in 0..limit {
        let age = diabetes.data[(i * diabetes.num_features)];
        let sex = diabetes.data[(i * diabetes.num_features) + 1];
        let bmi = diabetes.data[(i * diabetes.num_features) + 2];
        let bp = diabetes.data[(i * diabetes.num_features) + 3];
        let s1 = diabetes.data[(i * diabetes.num_features) + 4];
        let s2 = diabetes.data[(i * diabetes.num_features) + 5];
        let s3 = diabetes.data[(i * diabetes.num_features) + 6];
        let s4 = diabetes.data[(i * diabetes.num_features) + 7];
        let s5 = diabetes.data[(i * diabetes.num_features) + 8];
        let s6 = diabetes.data[(i * diabetes.num_features) + 9];
        let target = diabetes.target[i];
        run_with_args(
            "
        INSERT INTO pgml.diabetes (age, sex, bmi, bp, s1, s2, s3, s4, s5, s6, target) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ",
            vec![
                (PgBuiltInOids::FLOAT4OID.oid(), age.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), sex.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), bmi.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), bp.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), s1.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), s2.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), s3.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), s4.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), s5.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), s6.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), target.into_datum()),
            ],
        );
    }

    ("pgml.diabetes".to_string(), limit.try_into().unwrap())
}

pub fn load_digits(limit: Option<usize>) -> (String, i64) {
    let digits = smartcore::dataset::digits::load_dataset();
    Spi::run("DROP TABLE IF EXISTS pgml.digits");
    Spi::run("CREATE TABLE pgml.digits (image SMALLINT[][], target INTEGER)");
    // TODO replace run_with_args with upstream when PR is accepted
    run_with_args(
        "COMMENT ON TABLE pgml.digits IS '{description}'",
        vec![(
            PgBuiltInOids::TEXTOID.oid(),
            digits.description.into_datum(),
        )],
    );

    info!(
        "num_features: {}, num_samples: {}, feature_names: {:?}",
        digits.num_features, digits.num_samples, digits.feature_names
    );
    let limit = match limit {
        Some(limit) => {
            if limit > digits.num_samples {
                digits.num_samples
            } else {
                limit
            }
        }
        None => digits.num_samples,
    };
    let mut values = Vec::with_capacity(limit);
    for i in 0..limit {
        let target = digits.target[i];
        // shape the image in a 2d array
        let mut image = vec![vec![0.; 8]; 8];
        #[allow(clippy::needless_range_loop)] // x & y are in fact used
        for x in 0..8 {
            for y in 0..8 {
                image[x][y] = digits.data[(i * 64) + (x * 8) + y];
            }
        }
        // format the image into 2d SQL value
        let mut rows = Vec::with_capacity(8);
        for row in image {
            rows.push(
                "ARRAY[".to_string()
                    + &row
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                    + "]",
            );
        }
        let value = rows.join(",");
        values.push(format!("(ARRAY[{value}], {target})"));
    }
    let values = values.join(", ");
    let sql = format!("INSERT INTO pgml.digits (image, target) VALUES {values}");
    Spi::run(&sql);
    ("pgml.digits".to_string(), limit.try_into().unwrap())
}

pub fn load_iris(limit: Option<usize>) -> (String, i64) {
    let iris = smartcore::dataset::iris::load_dataset();
    Spi::run("DROP TABLE IF EXISTS pgml.iris");
    Spi::run(
        "CREATE TABLE pgml.iris (
        sepal_length FLOAT4, 
        sepal_width FLOAT4, 
        petal_length FLOAT4, 
        petal_width FLOAT4, 
        target INTEGER
    )",
    );
    // TODO replace run_with_args with upstream when PR is accepted
    run_with_args(
        "COMMENT ON TABLE pgml.iris IS '{description}'",
        vec![(PgBuiltInOids::TEXTOID.oid(), iris.description.into_datum())],
    );

    info!(
        "num_features: {}, num_samples: {}, feature_names: {:?}",
        iris.num_features, iris.num_samples, iris.feature_names
    );
    let limit = match limit {
        Some(limit) => {
            if limit > iris.num_samples {
                iris.num_samples
            } else {
                limit
            }
        }
        None => iris.num_samples,
    };
    for i in 0..limit {
        let sepal_length = iris.data[(i * iris.num_features)];
        let sepal_width = iris.data[(i * iris.num_features) + 1];
        let petal_length = iris.data[(i * iris.num_features) + 2];
        let petal_width = iris.data[(i * iris.num_features) + 3];
        let target = iris.target[i];
        run_with_args(
            "
        INSERT INTO pgml.iris (sepal_length, sepal_width, petal_length, petal_width, target)
        VALUES ($1, $2, $3, $4, $5)
        ",
            vec![
                (PgBuiltInOids::FLOAT4OID.oid(), sepal_length.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), sepal_width.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), petal_length.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), petal_width.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), target.into_datum()),
            ],
        );
    }

    ("pgml.iris".to_string(), limit.try_into().unwrap())
}

// TODO add upstream into smartcore
// pub fn load_linnerud(limit: Option<usize>) -> (String, i64) {
//     let linnerud = smartcore::dataset::linnerud::load_dataset();
//     Spi::run("DROP TABLE IF EXISTS pgml.linnerud");
//     Spi::run("CREATE TABLE pgml.linnerud(
//         chins FLOAT4,
//         situps FLOAT4,
//         jumps FLOAT4,
//         weight FLOAT4,
//         waste FLOAT4,
//         pulse FLOAT4
//     )");
//     // TODO replace run_with_args with upstream when PR is accepted
//     run_with_args(
//         "COMMENT ON TABLE pgml.linnerud IS '{description}'",
//         vec![(PgBuiltInOids::TEXTOID.oid(), linnerud.description.into_datum())],
//     );

//     info!("num_features: {}, num_samples: {}, feature_names: {:?}", linnerud.num_features, linnerud.num_samples, linnerud.feature_names);
//     let limit = match limit {
//         Some(limit) => if limit > linnerud.num_samples { linnerud.num_samples} else { limit },
//         None => linnerud.num_samples,
//     };
//     for i in 0..limit {
//         let chins = linnerud.data[(i * linnerud.num_features) + 0];
//         let situps = linnerud.data[(i * linnerud.num_features) + 1];
//         let jumps = linnerud.data[(i * linnerud.num_features) + 2];
//         let weight = linnerud.target[(i * linnerud.num_labels) + 0];
//         let waste = linnerud.target[(i * linnerud.num_labels) + 1];
//         let pulse = linnerud.target[(i * linnerud.num_labels) + 2];
//         run_with_args("
//         INSERT INTO pgml.iris (chins, situps, jumps, weight, waste, pulse)
//         VALUES ($1, $2, $3, $4, $5)
//         ", vec![
//             (PgBuiltInOids::FLOAT4OID.oid(), chins.into_datum()),
//             (PgBuiltInOids::FLOAT4OID.oid(), situps.into_datum()),
//             (PgBuiltInOids::FLOAT4OID.oid(), jumps.into_datum()),
//             (PgBuiltInOids::FLOAT4OID.oid(), weight.into_datum()),
//             (PgBuiltInOids::FLOAT4OID.oid(), waste.into_datum()),
//             (PgBuiltInOids::FLOAT4OID.oid(), pulse.into_datum()),
//         ]);
//     }

//     ("pgml.linnerud".to_string(), limit.try_into().unwrap())
// }

pub fn load_breast_cancer(limit: Option<usize>) -> (String, i64) {
    let breast_cancer = smartcore::dataset::breast_cancer::load_dataset();
    Spi::run("DROP TABLE IF EXISTS pgml.breast_cancer");
    Spi::run(
        r#"CREATE TABLE pgml.breast_cancer (
        "mean radius" FLOAT4, 
        "mean texture" FLOAT4, 
        "mean perimeter" FLOAT4, 
        "mean area" FLOAT4,
        "mean smoothness" FLOAT4,
        "mean compactness" FLOAT4,
        "mean concavity" FLOAT4,
        "mean concave points" FLOAT4,
        "mean symmetry" FLOAT4,
        "mean fractal dimension" FLOAT4,
        "radius error" FLOAT4,
        "texture error" FLOAT4,
        "perimeter error" FLOAT4,
        "area error" FLOAT4,
        "smoothness error" FLOAT4,
        "compactness error" FLOAT4,
        "concavity error" FLOAT4,
        "concave points error" FLOAT4,
        "symmetry error" FLOAT4,
        "fractal dimension error" FLOAT4,
        "worst radius" FLOAT4,
        "worst texture" FLOAT4,
        "worst perimeter" FLOAT4,
        "worst area" FLOAT4,
        "worst smoothness" FLOAT4,
        "worst compactness" FLOAT4,
        "worst concavity" FLOAT4,
        "worst concave points" FLOAT4,
        "worst symmetry" FLOAT4,
        "worst fractal dimension" FLOAT4,
        "malignant" BOOLEAN
    )"#,
    );
    // TODO replace run_with_args with upstream when PR is accepted
    run_with_args(
        "COMMENT ON TABLE pgml.breast_cancer IS '{description}'",
        vec![(
            PgBuiltInOids::TEXTOID.oid(),
            breast_cancer.description.into_datum(),
        )],
    );

    info!(
        "num_features: {}, num_samples: {}, feature_names: {:?}",
        breast_cancer.num_features, breast_cancer.num_samples, breast_cancer.feature_names
    );
    let limit = match limit {
        Some(limit) => {
            if limit > breast_cancer.num_samples {
                breast_cancer.num_samples
            } else {
                limit
            }
        }
        None => breast_cancer.num_samples,
    };
    for i in 0..limit {
        run_with_args(
            r#"
        INSERT INTO pgml.breast_cancer ("mean radius", "mean texture", "mean perimeter", "mean area", "mean smoothness", "mean compactness", "mean concavity", "mean concave points", "mean symmetry", 
            "mean fractal dimension", "radius error", "texture error", "perimeter error", "area error", "smoothness error", "compactness error", "concavity error", "concave points error", "symmetry error", 
            "fractal dimension error", "worst radius", "worst texture", "worst perimeter", "worst area", "worst smoothness", "worst compactness", "worst concavity", "worst concave points", "worst symmetry", 
            "worst fractal dimension", "malignant") 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)"#,
            vec![
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features)].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 1].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 2].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 3].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 4].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 5].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 6].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 7].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 8].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 9].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 10].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 11].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 12].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 13].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 14].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 15].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 16].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 17].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 18].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 19].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 20].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 21].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 22].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 23].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 24].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 25].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 26].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 27].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 28].into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    breast_cancer.data[(i * breast_cancer.num_features) + 29].into_datum(),
                ),
                (
                    PgBuiltInOids::BOOLOID.oid(),
                    (breast_cancer.target[i] == 0.).into_datum(),
                ),
            ],
        );
    }

    ("pgml.breast_cancer".to_string(), limit.try_into().unwrap())
}
