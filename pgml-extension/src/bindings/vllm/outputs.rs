use pyo3::prelude::*;

#[derive(Clone)]
pub struct RequestOutput {
    inner: PyObject,
}

#[derive(Clone)]
pub struct CompletionOutput {
    inner: PyObject,
}

impl RequestOutput {
    pub fn prompt(&self) -> PyResult<String> {
        Python::with_gil(|py| self.inner.getattr(py, "prompt")?.extract(py))
    }

    pub fn outputs(&self) -> PyResult<Vec<CompletionOutput>> {
        Python::with_gil(|py| self.inner.getattr(py, "outputs")?.extract(py))
    }
}

impl CompletionOutput {
    pub fn finished(&self) -> PyResult<bool> {
        Python::with_gil(|py| self.inner.getattr(py, "finished")?.call0(py)?.extract(py))
    }

    pub fn text(&self) -> PyResult<String> {
        Python::with_gil(|py| self.inner.getattr(py, "text")?.extract(py))
    }
}

impl<'source> FromPyObject<'source> for RequestOutput {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(Self {
            inner: ob.extract()?,
        })
    }
}

impl<'source> FromPyObject<'source> for CompletionOutput {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(Self {
            inner: ob.extract()?,
        })
    }
}
