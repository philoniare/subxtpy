use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use subxt::dynamic::{At, Value};
use subxt::{OnlineClient, PolkadotConfig};

#[pyclass]
struct SubxtClient {
    api: Arc<OnlineClient<PolkadotConfig>>,
}

#[pymethods]
impl SubxtClient {
    #[staticmethod]
    #[pyo3(name = "new")]
    fn py_new(py: Python<'_>) -> PyResult<&PyAny> {
        future_into_py(py, async {
            match OnlineClient::<PolkadotConfig>::new().await {
                Ok(api) => Ok(SubxtClient { api: Arc::new(api) }),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    e.to_string(),
                )),
            }
        })
    }

    fn fetch_free_balance<'py>(&self, py: Python<'py>, account: Vec<u8>) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let storage_query =
                subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(account)]);

            let result = api
                .storage()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .fetch(&storage_query)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let value = result
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>("Account not found")
                })?
                .to_value()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let free_balance = value.at("data").and_then(|v| v.at("free")).ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Free balance not found in account data",
                )
            })?;

            Ok(free_balance.to_string())
        })
    }
}

#[pymodule]
fn subxtpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SubxtClient>()?;
    Ok(())
}
