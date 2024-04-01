use std::fmt::{Display, Formatter};

use flate2::read::GzDecoder;
use pgrx::*;
use serde::Deserialize;

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
            "Dataset {{ num_features: {}, num_labels: {}, num_distinct_labels: {}, num_rows: {}, num_train_rows: {}, num_test_rows: {} }}",
            self.num_features, self.num_labels, self.num_distinct_labels, self.num_rows, self.num_train_rows, self.num_test_rows,
        )
    }
}

impl Dataset {
    pub fn fold(&self, k: usize, folds: usize) -> Dataset {
        if folds < 2 {
            error!("It doesn't make sense to have k folds < 2. Use the dataset train/test split directly instead.");
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

pub enum TextDatasetType {
    TextClassification(TextClassificationDataset),
    TextPairClassification(TextPairClassificationDataset),
    Conversation(ConversationDataset),
}

impl TextDatasetType {
    pub fn num_features(&self) -> usize {
        match self {
            TextDatasetType::TextClassification(dataset) => dataset.num_features,
            TextDatasetType::TextPairClassification(dataset) => dataset.num_features,
            TextDatasetType::Conversation(dataset) => dataset.num_features,
        }
    }
}

// TextClassificationDataset
pub struct TextClassificationDataset {
    pub text_train: Vec<String>,
    pub class_train: Vec<String>,
    pub text_test: Vec<String>,
    pub class_test: Vec<String>,
    pub num_features: usize,
    pub num_labels: usize,
    pub num_rows: usize,
    pub num_train_rows: usize,
    pub num_test_rows: usize,
    pub num_distinct_labels: usize,
}

impl Display for TextClassificationDataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "TextClassificationDataset {{ num_distinct_labels: {}, num_rows: {}, num_train_rows: {}, num_test_rows: {} }}",
            self.num_distinct_labels, self.num_rows, self.num_train_rows, self.num_test_rows,
        )
    }
}

pub struct TextPairClassificationDataset {
    pub text1_train: Vec<String>,
    pub text2_train: Vec<String>,
    pub class_train: Vec<String>,
    pub text1_test: Vec<String>,
    pub text2_test: Vec<String>,
    pub class_test: Vec<String>,
    pub num_features: usize,
    pub num_labels: usize,
    pub num_rows: usize,
    pub num_train_rows: usize,
    pub num_test_rows: usize,
    pub num_distinct_labels: usize,
}

impl Display for TextPairClassificationDataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "TextPairClassificationDataset {{ num_distinct_labels: {}, num_rows: {}, num_train_rows: {}, num_test_rows: {} }}",
            self.num_distinct_labels, self.num_rows, self.num_train_rows, self.num_test_rows,
        )
    }
}

pub struct ConversationDataset {
    pub system_train: Vec<String>,
    pub user_train: Vec<String>,
    pub assistant_train: Vec<String>,
    pub system_test: Vec<String>,
    pub user_test: Vec<String>,
    pub assistant_test: Vec<String>,
    pub num_features: usize,
    pub num_rows: usize,
    pub num_train_rows: usize,
    pub num_test_rows: usize,
}

impl Display for ConversationDataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "ConversationDataset {{ num_rows: {}, num_train_rows: {}, num_test_rows: {} }}",
            self.num_rows, self.num_train_rows, self.num_test_rows,
        )
    }
}
fn drop_table_if_exists(table_name: &str) {
    // Avoid the existence for DROP TABLE IF EXISTS warning by checking the schema for the table first
    let table_count = Spi::get_one_with_args::<i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = 'pgml'",
        vec![(PgBuiltInOids::TEXTOID.oid(), table_name.into_datum())],
    )
    .unwrap()
    .unwrap();
    if table_count == 1 {
        Spi::run(&format!(r#"DROP TABLE pgml.{table_name} CASCADE"#)).unwrap();
    }
}

#[derive(Deserialize)]
struct BreastCancerRow {
    mean_radius: f32,
    mean_texture: f32,
    mean_perimeter: f32,
    mean_area: f32,
    mean_smoothness: f32,
    mean_compactness: f32,
    mean_concavity: f32,
    mean_concave_points: f32,
    mean_symmetry: f32,
    mean_fractal_dimension: f32,
    radius_error: f32,
    texture_error: f32,
    perimeter_error: f32,
    area_error: f32,
    smoothness_error: f32,
    compactness_error: f32,
    concavity_error: f32,
    concave_points_error: f32,
    symmetry_error: f32,
    fractal_dimension_error: f32,
    worst_radius: f32,
    worst_texture: f32,
    worst_perimeter: f32,
    worst_area: f32,
    worst_smoothness: f32,
    worst_compactness: f32,
    worst_concavity: f32,
    worst_concave_points: f32,
    worst_symmetry: f32,
    worst_fractal_dimension: f32,
    target: i32,
}

pub fn load_breast_cancer(limit: Option<usize>) -> (String, i64) {
    drop_table_if_exists("breast_cancer");
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
    )
    .unwrap();

    let limit = match limit {
        Some(limit) => limit,
        None => usize::MAX,
    };

    let data: &[u8] = std::include_bytes!("datasets/breast_cancer.csv.gz");
    let decoder = GzDecoder::new(data);
    let mut csv = csv::ReaderBuilder::new().from_reader(decoder);

    let mut inserted = 0;
    for (i, row) in csv.deserialize().enumerate() {
        if i >= limit {
            break;
        }
        let row: BreastCancerRow = row.unwrap();
        Spi::run_with_args(
            r#"
        INSERT INTO pgml.breast_cancer ("mean radius", "mean texture", "mean perimeter", "mean area", "mean smoothness", "mean compactness", "mean concavity", "mean concave points", "mean symmetry", 
            "mean fractal dimension", "radius error", "texture error", "perimeter error", "area error", "smoothness error", "compactness error", "concavity error", "concave points error", "symmetry error", 
            "fractal dimension error", "worst radius", "worst texture", "worst perimeter", "worst area", "worst smoothness", "worst compactness", "worst concavity", "worst concave points", "worst symmetry", 
            "worst fractal dimension", "malignant") 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)"#,
            Some(vec![
                (PgBuiltInOids::FLOAT4OID.oid(), row.mean_radius.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_texture.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_perimeter.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.mean_area.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_smoothness.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_compactness.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_concavity.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_concave_points.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_symmetry.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.mean_fractal_dimension.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.radius_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.texture_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.perimeter_error.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.area_error.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.smoothness_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.compactness_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.concavity_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.concave_points_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.symmetry_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.fractal_dimension_error.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_radius.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_texture.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_perimeter.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.worst_area.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_smoothness.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_compactness.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_concavity.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_concave_points.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_symmetry.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.worst_fractal_dimension.into_datum(),
                ),
                (PgBuiltInOids::BOOLOID.oid(), (row.target == 0).into_datum()),
            ]),
        ).unwrap();
        inserted += 1;
    }

    ("pgml.breast_cancer".to_string(), inserted)
}

#[derive(Deserialize)]
struct DiabetesRow {
    age: f32,
    sex: f32,
    bmi: f32,
    bp: f32,
    s1: f32,
    s2: f32,
    s3: f32,
    s4: f32,
    s5: f32,
    s6: f32,
    target: f32,
}

pub fn load_diabetes(limit: Option<usize>) -> (String, i64) {
    drop_table_if_exists("diabetes");
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
        target FLOAT4
    )",
    )
    .unwrap();

    let limit = match limit {
        Some(limit) => limit,
        None => usize::MAX,
    };

    let data: &[u8] = std::include_bytes!("datasets/diabetes.csv.gz");
    let decoder = GzDecoder::new(data);
    let mut csv = csv::ReaderBuilder::new().from_reader(decoder);

    let mut inserted = 0;
    for (i, row) in csv.deserialize().enumerate() {
        if i >= limit {
            break;
        }
        let row: DiabetesRow = row.unwrap();
        Spi::run_with_args(
            "
        INSERT INTO pgml.diabetes (age, sex, bmi, bp, s1, s2, s3, s4, s5, s6, target) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ",
            Some(vec![
                (PgBuiltInOids::FLOAT4OID.oid(), row.age.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.sex.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.bmi.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.bp.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.s1.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.s2.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.s3.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.s4.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.s5.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.s6.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.target.into_datum()),
            ]),
        )
        .unwrap();
        inserted += 1;
    }

    ("pgml.diabetes".to_string(), inserted)
}

#[derive(Deserialize)]
struct DigitsRow {
    image: String,
    target: i16,
}

pub fn load_digits(limit: Option<usize>) -> (String, i64) {
    drop_table_if_exists("digits");
    Spi::run("CREATE TABLE pgml.digits (image SMALLINT[][], target SMALLINT)").unwrap();

    let limit = match limit {
        Some(limit) => limit,
        None => usize::MAX,
    };

    let data: &[u8] = std::include_bytes!("datasets/digits.csv.gz");
    let decoder = GzDecoder::new(data);
    let mut csv = csv::ReaderBuilder::new().from_reader(decoder);

    let mut inserted = 0;
    for (i, row) in csv.deserialize().enumerate() {
        if i >= limit {
            break;
        }
        let row: DigitsRow = row.unwrap();
        Spi::run_with_args(
            "
            INSERT INTO pgml.digits (image, target)
            VALUES ($1::SMALLINT[][], $2)
            ",
            Some(vec![
                (PgBuiltInOids::TEXTOID.oid(), row.image.into_datum()),
                (PgBuiltInOids::INT2OID.oid(), row.target.into_datum()),
            ]),
        )
        .unwrap();
        inserted += 1;
    }

    ("pgml.digits".to_string(), inserted)
}

#[derive(Deserialize)]
struct IrisRow {
    sepal_length: f32,
    sepal_width: f32,
    petal_length: f32,
    petal_width: f32,
    target: i32,
}

pub fn load_iris(limit: Option<usize>) -> (String, i64) {
    drop_table_if_exists("iris");
    Spi::run(
        "CREATE TABLE pgml.iris (
        sepal_length FLOAT4, 
        sepal_width FLOAT4, 
        petal_length FLOAT4, 
        petal_width FLOAT4, 
        target INT4
    )",
    )
    .unwrap();

    let limit = match limit {
        Some(limit) => limit,
        None => usize::MAX,
    };

    let data: &[u8] = std::include_bytes!("datasets/iris.csv.gz");
    let decoder = GzDecoder::new(data);
    let mut csv = csv::ReaderBuilder::new().from_reader(decoder);

    let mut inserted = 0;
    for (i, row) in csv.deserialize().enumerate() {
        if i >= limit {
            break;
        }
        let row: IrisRow = row.unwrap();
        Spi::run_with_args(
            "
        INSERT INTO pgml.iris (sepal_length, sepal_width, petal_length, petal_width, target)
        VALUES ($1, $2, $3, $4, $5)
        ",
            Some(vec![
                (PgBuiltInOids::FLOAT4OID.oid(), row.sepal_length.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.sepal_width.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.petal_length.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.petal_width.into_datum()),
                (PgBuiltInOids::INT4OID.oid(), row.target.into_datum()),
            ]),
        )
        .unwrap();
        inserted += 1;
    }

    ("pgml.iris".to_string(), inserted)
}

#[derive(Deserialize)]
struct LinnerudRow {
    chins: f32,
    situps: f32,
    jumps: f32,
    weight: f32,
    waist: f32,
    pulse: f32,
}

pub fn load_linnerud(limit: Option<usize>) -> (String, i64) {
    drop_table_if_exists("linnerud");
    Spi::run(
        "CREATE TABLE pgml.linnerud(
        chins FLOAT4,
        situps FLOAT4,
        jumps FLOAT4,
        weight FLOAT4,
        waist FLOAT4,
        pulse FLOAT4
    )",
    )
    .unwrap();

    let limit = match limit {
        Some(limit) => limit,
        None => usize::MAX,
    };

    let data: &[u8] = std::include_bytes!("datasets/linnerud.csv.gz");
    let decoder = GzDecoder::new(data);
    let mut csv = csv::ReaderBuilder::new().from_reader(decoder);

    let mut inserted = 0;
    for (i, row) in csv.deserialize().enumerate() {
        if i >= limit {
            break;
        }
        let row: LinnerudRow = row.unwrap();
        Spi::run_with_args(
            "
        INSERT INTO pgml.linnerud (chins, situps, jumps, weight, waist, pulse)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
            Some(vec![
                (PgBuiltInOids::FLOAT4OID.oid(), row.chins.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.situps.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.jumps.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.weight.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.waist.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.pulse.into_datum()),
            ]),
        )
        .unwrap();
        inserted += 1;
    }

    ("pgml.linnerud".to_string(), inserted)
}

#[derive(Deserialize)]
struct WineRow {
    alcohol: f32,
    malic_acid: f32,
    ash: f32,
    alcalinity_of_ash: f32,
    magnesium: f32,
    total_phenols: f32,
    flavanoids: f32,
    nonflavanoid_phenols: f32,
    proanthocyanins: f32,
    hue: f32,
    color_intensity: f32,
    od280_od315_of_diluted_wines: f32,
    proline: f32,
    target: i32,
}

pub fn load_wine(limit: Option<usize>) -> (String, i64) {
    drop_table_if_exists("wine");
    Spi::run(
        r#"CREATE TABLE pgml.wine (
            alcohol FLOAT4,
            malic_acid FLOAT4,
            ash FLOAT4,
            alcalinity_of_ash FLOAT4,
            magnesium FLOAT4,
            total_phenols FLOAT4,
            flavanoids FLOAT4,
            nonflavanoid_phenols FLOAT4,
            proanthocyanins FLOAT4,
            hue FLOAT4,
            color_intensity FLOAT4,
            "od280/od315_of_diluted_wines" FLOAT4,
            proline FLOAT4,
            target INT4
        )"#,
    )
    .unwrap();

    let limit = match limit {
        Some(limit) => limit,
        None => usize::MAX,
    };

    let data: &[u8] = std::include_bytes!("datasets/wine.csv.gz");
    let decoder = GzDecoder::new(data);
    let mut csv = csv::ReaderBuilder::new().from_reader(decoder);

    let mut inserted = 0;
    for (i, row) in csv.deserialize().enumerate() {
        if i >= limit {
            break;
        }
        let row: WineRow = row.unwrap();
        Spi::run_with_args(
            r#"
        INSERT INTO pgml.wine (alcohol, malic_acid, ash, alcalinity_of_ash, magnesium, total_phenols, flavanoids, nonflavanoid_phenols, proanthocyanins, color_intensity, hue, "od280/od315_of_diluted_wines", proline, target) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
            Some(vec![
                (PgBuiltInOids::FLOAT4OID.oid(), row.alcohol.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.malic_acid.into_datum()),
                (PgBuiltInOids::FLOAT4OID.oid(), row.ash.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.alcalinity_of_ash.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.magnesium.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.total_phenols.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.flavanoids.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.nonflavanoid_phenols.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.proanthocyanins.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.hue.into_datum()),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.color_intensity.into_datum(),
                ),
                (
                    PgBuiltInOids::FLOAT4OID.oid(),
                    row.od280_od315_of_diluted_wines.into_datum(),
                ),
                (PgBuiltInOids::FLOAT4OID.oid(), row.proline.into_datum()),
                (PgBuiltInOids::INT4OID.oid(), row.target.into_datum()),
            ]),
        ).unwrap();
        inserted += 1;
    }

    ("pgml.wine".to_string(), inserted)
}
