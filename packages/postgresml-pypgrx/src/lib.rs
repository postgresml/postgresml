use pgrx::*;
use pgrx_pg_sys::info;
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn insert_logs(project_id: i64, model_id: i64, logs: String) -> PyResult<String> {
    let id_value = Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml.logs (model_id, project_id, logs) VALUES ($1, $2, $3::JSONB) RETURNING id;",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
            (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), logs.into_datum()),
        ],
    )
    .unwrap()
    .unwrap();

    Ok(format!("Inserted logs with id: {}", id_value))
}

#[pyfunction]
fn print_info(info: String) -> PyResult<String> {
    info!("{}", info);
    Ok(info)
}
/// A Python module implemented in Rust.
#[pymodule]
fn pypgrx(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_info, m)?)?;
    m.add_function(wrap_pyfunction!(insert_logs, m)?)?;
    Ok(())
}
