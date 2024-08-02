use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use subxt::{OnlineClient, PolkadotConfig};
use subxt::dynamic::{Value};
use subxt::ext::scale_value::{ValueDef, Primitive, Composite};
use pyo3::types::{PyDict, PyList, PyBytes};
use subxt::backend::StreamOfResults;
use subxt::storage::{StorageKeyValuePair, DynamicAddress};
use subxt_signer::sr25519::dev;
use subxt::config::polkadot::PolkadotExtrinsicParamsBuilder as Params;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod polkadot {}

#[pyclass]
struct StorageIterator {
    results: Arc<tokio::sync::Mutex<StreamOfResults<StorageKeyValuePair<DynamicAddress<Vec<Value>>>>>>,
}

#[pymethods]
impl StorageIterator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'a>(&self, py: Python<'a>) -> PyResult<Option<PyObject>> {
        let results = self.results.clone();
        let future = pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut results = results.lock().await;
            if let Some(result) = results.next().await {
                let key_val = result.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                let py_dict = Python::with_gil(|py| {
                    let dict = PyDict::new(py);
                    dict.set_item("key_bytes", PyBytes::new(py, &key_val.key_bytes))?;

                    let py_keys = PyList::new(py, key_val.keys.iter().map(|k| {
                        let new_k = k.clone().map_context(|_| 0u32);
                        decoded_value_to_py_object(py, &new_k).unwrap()
                    }));
                    dict.set_item("keys", py_keys)?;

                    // Convert value to PyObject
                    let py_value = decoded_value_to_py_object(py, &key_val.value.to_value().unwrap())?;
                    dict.set_item("value", py_value)?;
                    Ok::<PyObject, PyErr>(dict.to_object(py))
                })?;
                Ok(Some(py_dict))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyStopAsyncIteration, _>("Iterator exhausted"))
            }
        });
        Ok(Some(future?.into()))
    }
}

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

    #[staticmethod]
    #[pyo3(name = "from_url")]
    fn from_url(py: Python<'_>, url: String) -> PyResult<&PyAny> {
        future_into_py(py, async {
            match OnlineClient::<PolkadotConfig>::from_url(url).await {
                Ok(api) => Ok(SubxtClient { api: Arc::new(api) }),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    e.to_string(),
                )),
            }
        })
    }

    fn storage<'py>(
        &self,
        py: Python<'py>,
        pallet_name: String,
        entry_name: String,
        key: Vec<u8>,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let storage_query =
                subxt::dynamic::storage(pallet_name, entry_name, vec![Value::from_bytes(key)]);

            let result = api
                .storage()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .fetch(&storage_query)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            match result {
                Some(value) => {
                    let decoded = value.to_value().map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                    })?;
                    let py_value = Python::with_gil(|py| decoded_value_to_py_object(py, &decoded))?;
                    Ok(py_value)
                }
                None => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Storage not found",
                )),
            }
        })
    }

    fn constant<'py>(
        &self,
        py: Python<'py>,
        pallet_name: String,
        constant_name: String,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let constant_query = subxt::dynamic::constant(pallet_name, constant_name);

            let value = api
                .constants()
                .at(&constant_query)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let decoded = value.to_value().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?;
            let py_value = Python::with_gil(|py| decoded_value_to_py_object(py, &decoded))?;
            Ok(py_value)
        })
    }

    fn events<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let events = api
                .events()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let events_vec: Vec<_> = events.iter().collect::<Result<Vec<_>, _>>()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let py_events: PyResult<PyObject> = Python::with_gil(|py| {
                let py_list = PyList::new(py, events_vec.iter().map(|event| {
                    let py_event = PyDict::new(py);
                    py_event.set_item("pallet", event.pallet_name()).unwrap();
                    py_event.set_item("variant", event.variant_name()).unwrap();
                    py_event.set_item(
                        "fields",
                        composite_to_py_object(py, &event.field_values().unwrap()).unwrap(),
                    ).unwrap();
                    py_event.to_object(py)
                }));
                Ok(py_list.into())
            });
            py_events
        })
    }

    fn runtime_api_call<'py>(
        &self,
        py: Python<'py>,
        pallet_name: String,
        entry_name: String,
        account: Vec<u8>,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let runtime_api_call = subxt::dynamic::runtime_api_call(
                &pallet_name,
                &entry_name,
                vec![Value::from_bytes(account)],
            );

            let nonce = api
                .runtime_api()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .call(runtime_api_call)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let decoded_nonce = nonce.to_value().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?;
            let py_value = Python::with_gil(|py| decoded_value_to_py_object(py, &decoded_nonce))?;
            Ok(py_value)
        })
    }

    fn storage_iter<'py>(
        &self,
        py: Python<'py>,
        pallet_name: String,
        entry_name: String,
        key: Vec<u8>,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let storage_query = subxt::dynamic::storage(pallet_name, entry_name, vec![Value::from_bytes(key)]);

            let results = api
                .storage()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .iter(storage_query)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok(StorageIterator { results: Arc::new(tokio::sync::Mutex::new(results)) })
        })
    }

    fn sign_and_submit<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let dest = dev::bob().public_key().into();
            let balance_transfer_tx = polkadot::tx().balances().transfer_allow_death(dest, 10_000);
            let from = dev::alice();
            let tx_params = Params::new().build();
            let hash = api.tx().sign_and_submit(&balance_transfer_tx, &from, tx_params).await.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(hash.0)
        })
    }
}

fn composite_to_py_object(py: Python, composite: &Composite<u32>) -> PyResult<PyObject> {
    let py_dict = PyDict::new(py);

    match composite {
        Composite::Named(named) => {
            for (key, value) in named.iter() {
                let py_value = decoded_value_to_py_object(py, value)?;
                py_dict.set_item(key, py_value)?;
            }
        }
        Composite::Unnamed(unnamed) => {
            for (index, value) in unnamed.iter().enumerate() {
                let py_value = decoded_value_to_py_object(py, value)?;
                py_dict.set_item(index.to_string(), py_value)?;
            }
        }
    }
    Ok(py_dict.into())
}

fn primitive_to_py_object(py: Python, primitive: &Primitive) -> PyResult<PyObject> {
    match primitive {
        Primitive::Bool(b) => Ok(b.to_object(py)),
        Primitive::Char(c) => Ok(c.to_object(py)),
        Primitive::String(s) => Ok(s.to_object(py)),
        Primitive::U128(u) => Ok(u.to_object(py)),
        Primitive::I128(i) => Ok(i.to_object(py)),
        _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Unsupported primitive type")),
    }
}

fn decoded_value_to_py_object(py: Python, decoded_value: &Value<u32>) -> PyResult<PyObject> {
    match &decoded_value.value {
        ValueDef::Composite(composite) => composite_to_py_object(py, composite),
        ValueDef::Variant(variant) => {
            let py_dict = PyDict::new(py);
            py_dict.set_item("variant_name", variant.name.clone())?;

            match &variant.values {
                Composite::Named(named) => {
                    let py_values = PyDict::new(py);
                    for (key, value) in named.iter() {
                        let py_value = decoded_value_to_py_object(py, value)?;
                        py_values.set_item(key, py_value)?;
                    }
                    py_dict.set_item("values", py_values)?;
                }
                Composite::Unnamed(unnamed) => {
                    let py_values = PyList::new(py, unnamed.iter().map(|v| decoded_value_to_py_object(py, v).unwrap()));
                    py_dict.set_item("values", py_values)?;
                }
            }

            Ok(py_dict.into())
        }
        ValueDef::BitSequence(bit_sequence) => {
            let bits: Vec<bool> = bit_sequence.iter().collect();
            Ok(PyList::new(py, bits).into())
        }
        ValueDef::Primitive(primitive) => primitive_to_py_object(py, primitive)
    }
}

#[pymodule]
fn subxtpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SubxtClient>()?;
    m.add_class::<StorageIterator>()?;
    Ok(())
}
