use hex;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyString};
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use subxt::backend::StreamOfResults;
use subxt::blocks::Block;
use subxt::config::polkadot::PolkadotExtrinsicParamsBuilder as Params;
use subxt::dynamic::{tx, Value};
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};
use subxt::storage::{DynamicAddress, StorageKeyValuePair};
use subxt::tx::Signer as SignerT;
use subxt::Config;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::{Keypair as STKeypair, PublicKey, Signature};

#[derive(Clone)]
enum AddressUse {
    Storage,
    Extrinsic,
}

/// A Keypair used for signing transactions.
///
/// This class wraps an sr25519 keypair and provides methods to create a keypair from a secret key.
#[pyclass]
#[derive(Clone)]
struct Keypair {
    keypair: STKeypair,
}

#[pymethods]
impl Keypair {
    /// Create a new Keypair from a secret key in hexadecimal format.
    ///
    /// Args:
    ///     secret_key (str): A 64-character hexadecimal string representing the secret key.
    ///
    /// Returns:
    ///     Keypair: A new Keypair instance.
    ///
    /// Raises:
    ///     ValueError: If the secret key is not 64 hex characters long or invalid.
    #[staticmethod]
    fn from_secret_key(_py: Python, secret_key: &str) -> PyResult<Self> {
        if secret_key.len() != 64 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Secret key must be 32 bytes (64 hex characters) long",
            ));
        }

        let mut secret_key_bytes: [u8; 32] = [0; 32];
        hex::decode_to_slice(secret_key, &mut secret_key_bytes).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid hex string: {}", e))
        })?;

        let keypair = STKeypair::from_secret_key(secret_key_bytes)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(Keypair { keypair })
    }
}

impl<T: Config> SignerT<T> for Keypair
where
    T::AccountId: From<PublicKey>,
    T::Address: From<PublicKey>,
    T::Signature: From<Signature>,
{
    fn account_id(&self) -> T::AccountId {
        self.keypair.public_key().into()
    }

    fn address(&self) -> T::Address {
        self.keypair.public_key().into()
    }

    fn sign(&self, signer_payload: &[u8]) -> T::Signature {
        self.keypair.sign(signer_payload).into()
    }
}

/// A subscription to new blocks on the blockchain.
///
/// This class provides an asynchronous iterator over new blocks as they are finalized.
///
/// Example:
///
/// ```python
/// async for block in subscription:
///     print(block)
/// ```
#[pyclass]
struct BlockSubscription {
    blocks_stream: Arc<
        tokio::sync::Mutex<StreamOfResults<Block<PolkadotConfig, OnlineClient<PolkadotConfig>>>>,
    >,
}

#[pymethods]
impl BlockSubscription {
    /// Return the asynchronous iterator object.
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Return the next block from the subscription.
    ///
    /// Yields:
    ///     dict: A dictionary containing the block number, block hash, and a list of extrinsics.
    ///
    /// Raises:
    ///     StopAsyncIteration: When no more blocks are available.
    fn __anext__<'a>(&self, py: Python<'a>) -> PyResult<Option<PyObject>> {
        let blocks_stream = self.blocks_stream.clone();
        let future = future_into_py(py, async move {
            let mut blocks_stream = blocks_stream.lock().await;
            match blocks_stream.next().await {
                Some(block_result) => {
                    let block = block_result.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                    })?;

                    // Get block number and hash
                    let block_number = block.header().number;
                    let block_hash = block.hash();

                    // Get the extrinsics
                    let extrinsics = block.extrinsics().await.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                    })?;

                    let mut extrinsics_info = vec![];

                    for ext_result in extrinsics.iter() {
                        let ext = ext_result.map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                        })?;

                        // Get signed extensions
                        let signed_extensions = match ext.signed_extensions() {
                            Some(signed_extensions) => signed_extensions,
                            None => continue, // Skip unsigned extrinsics
                        };

                        // Get metadata
                        let meta = ext.extrinsic_metadata().map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                        })?;
                        let pallet_name = meta.pallet.name();
                        let call_name = &meta.variant.name;

                        // Get field values
                        let fields = ext.field_values().map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                        })?;

                        // Convert fields to Python object
                        let py_fields = Python::with_gil(|py| composite_to_py_object(py, &fields))?;

                        // Collect signed extensions
                        let py_signed_extensions = Python::with_gil(|py| -> PyResult<PyObject> {
                            let mut ext_list = Vec::new();
                            for se_result in signed_extensions.iter() {
                                let se = match se_result {
                                    Ok(se) => se,
                                    Err(_) => continue,
                                };
                                let name = se.name();
                                if ["CheckMortality", "CheckNonce", "ChargeTransactionPayment"]
                                    .contains(&name)
                                {
                                    let value = match se.value() {
                                        Ok(value) => value,
                                        Err(_) => continue,
                                    };
                                    let value_py = decoded_value_to_py_object(py, &value)?;
                                    let dict = PyDict::new(py);
                                    dict.set_item("name", name)?;
                                    dict.set_item("value", value_py)?;
                                    ext_list.push(dict.to_object(py));
                                }
                            }
                            Ok(PyList::new(py, ext_list).to_object(py))
                        })?;

                        // Collect extrinsic information
                        let extrinsic_info = Python::with_gil(|py| -> PyResult<PyObject> {
                            let dict = PyDict::new(py);
                            dict.set_item("pallet", pallet_name)?;
                            dict.set_item("call", call_name)?;
                            dict.set_item("fields", py_fields)?;
                            dict.set_item("signed_extensions", py_signed_extensions)?;
                            Ok(dict.to_object(py))
                        })?;

                        extrinsics_info.push(extrinsic_info);
                    }

                    // Create a Python dictionary with block info
                    let py_block_info = Python::with_gil(|py| -> PyResult<PyObject> {
                        let dict = PyDict::new(py);
                        dict.set_item("block_number", block_number)?;
                        dict.set_item("block_hash", format!("{:?}", block_hash))?;
                        dict.set_item("extrinsics", PyList::new(py, extrinsics_info))?;
                        Ok(dict.to_object(py))
                    })?;
                    Ok(Some(py_block_info))
                }
                None => Err(PyErr::new::<pyo3::exceptions::PyStopAsyncIteration, _>(
                    "No more blocks",
                )),
            }
        });
        Ok(Some(future?.into()))
    }
}

/// An asynchronous iterator over storage key-value pairs.
///
/// This class allows you to iterate over storage entries retrieved from the blockchain.
#[pyclass]
struct StorageIterator {
    results:
        Arc<tokio::sync::Mutex<StreamOfResults<StorageKeyValuePair<DynamicAddress<Vec<Value>>>>>>,
}

#[pymethods]
impl StorageIterator {
    /// Return the asynchronous iterator object.
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Return the next storage key-value pair from the iterator.
    ///
    /// Yields:
    ///     dict: A dictionary containing the key bytes, keys, and value.
    ///
    /// Raises:
    ///     StopAsyncIteration: When no more items are available.
    fn __anext__<'a>(&self, py: Python<'a>) -> PyResult<Option<PyObject>> {
        let results = self.results.clone();
        let future = future_into_py(py, async move {
            let mut results = results.lock().await;
            if let Some(result) = results.next().await {
                let key_val = result.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;
                let py_dict = Python::with_gil(|py| -> PyResult<PyObject> {
                    let dict = PyDict::new(py);
                    dict.set_item("key_bytes", PyBytes::new(py, &key_val.key_bytes))?;

                    let py_keys = PyList::new(
                        py,
                        key_val.keys.iter().map(|k| {
                            let new_k = k.clone().map_context(|_| 0u32);
                            decoded_value_to_py_object(py, &new_k).unwrap()
                        }),
                    );
                    dict.set_item("keys", py_keys)?;

                    // Convert value to PyObject
                    let py_value =
                        decoded_value_to_py_object(py, &key_val.value.to_value().unwrap())?;
                    dict.set_item("value", py_value)?;
                    Ok(dict.to_object(py))
                })?;
                Ok(Some(py_dict))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyStopAsyncIteration, _>(
                    "Iterator exhausted",
                ))
            }
        });
        Ok(Some(future?.into()))
    }
}

/// A client for interacting with a Substrate-based blockchain.
///
/// This class provides methods to interact with the blockchain, including fetching storage entries,
/// constants, events, making runtime API calls, and submitting transactions.
#[pyclass]
struct SubxtClient {
    api: Arc<OnlineClient<PolkadotConfig>>,
}

#[pymethods]
impl SubxtClient {
    /// Create a new SubxtClient instance asynchronously.
    ///
    /// Returns:
    ///     SubxtClient: A new client connected to the default network.
    ///
    /// Raises:
    ///     RuntimeError: If the client fails to connect.
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

    /// Create a new SubxtClient instance from a URL asynchronously.
    ///
    /// Args:
    ///     url (str): The URL of the node to connect to.
    ///
    /// Returns:
    ///     SubxtClient: A new client connected to the specified node.
    ///
    /// Raises:
    ///     RuntimeError: If the client fails to connect.
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

    /// Fetch a storage entry from the blockchain asynchronously.
    ///
    /// Args:
    ///     pallet_name (str): The name of the pallet.
    ///     entry_name (str): The name of the storage entry.
    ///     key (list): A list of keys for the storage entry.
    ///
    /// Returns:
    ///     Any: The value of the storage entry.
    ///
    /// Raises:
    ///     RuntimeError: If fetching the storage entry fails.
    ///     ValueError: If the storage entry is not found.
    fn storage<'py>(
        &self,
        py: Python<'py>,
        pallet_name: String,
        entry_name: String,
        key: &PyList,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        let values: Vec<Value> = key
            .iter()
            .map(|item| py_object_to_value(item, AddressUse::Storage))
            .collect::<PyResult<Vec<Value>>>()?;
        future_into_py(py, async move {
            let storage_query = subxt::dynamic::storage(pallet_name, entry_name, values);
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

    /// Fetch a constant value from the blockchain asynchronously.
    ///
    /// Args:
    ///     pallet_name (str): The name of the pallet.
    ///     constant_name (str): The name of the constant.
    ///
    /// Returns:
    ///     Any: The value of the constant.
    ///
    /// Raises:
    ///     RuntimeError: If fetching the constant fails.
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

            let decoded = value
                .to_value()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let py_value = Python::with_gil(|py| decoded_value_to_py_object(py, &decoded))?;
            Ok(py_value)
        })
    }

    /// Fetch events from the blockchain asynchronously.
    ///
    /// Returns:
    ///     list: A list of events, where each event is a dictionary containing pallet, variant, and fields.
    ///
    /// Raises:
    ///     RuntimeError: If fetching events fails.
    fn events<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        future_into_py(py, async move {
            let events =
                api.events().at_latest().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;

            let events_vec: Vec<_> = events
                .iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let py_events: PyResult<PyObject> = Python::with_gil(|py| {
                let py_list = PyList::new(
                    py,
                    events_vec.iter().map(|event| {
                        let py_event = PyDict::new(py);
                        py_event.set_item("pallet", event.pallet_name()).unwrap();
                        py_event.set_item("variant", event.variant_name()).unwrap();
                        py_event
                            .set_item(
                                "fields",
                                composite_to_py_object(py, &event.field_values().unwrap()).unwrap(),
                            )
                            .unwrap();
                        py_event.to_object(py)
                    }),
                );
                Ok(py_list.into())
            });
            py_events
        })
    }

    /// Perform a runtime API call to the blockchain asynchronously.
    ///
    /// Args:
    ///     pallet_name (str): The name of the pallet.
    ///     entry_name (str): The name of the runtime API function.
    ///     key (list): A list of arguments for the runtime API call.
    ///
    /// Returns:
    ///     Any: The result of the runtime API call.
    ///
    /// Raises:
    ///     RuntimeError: If the runtime API call fails.
    fn runtime_api_call<'py>(
        &self,
        py: Python<'py>,
        pallet_name: String,
        entry_name: String,
        key: &PyList,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        let values: Vec<Value> = key
            .iter()
            .map(|item| py_object_to_value(item, AddressUse::Storage))
            .collect::<PyResult<Vec<Value>>>()?;
        future_into_py(py, async move {
            let runtime_api_call =
                subxt::dynamic::runtime_api_call(&pallet_name, &entry_name, values);

            let result = api
                .runtime_api()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .call(runtime_api_call)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let decoded = result
                .to_value()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let py_value = Python::with_gil(|py| decoded_value_to_py_object(py, &decoded))?;
            Ok(py_value)
        })
    }

    /// Iterate over storage entries from the blockchain asynchronously.
    ///
    /// Args:
    ///     pallet_name (str): The name of the pallet.
    ///     entry_name (str): The name of the storage entry.
    ///     key (bytes): The prefix key for iteration.
    ///
    /// Returns:
    ///     StorageIterator: An asynchronous iterator over storage key-value pairs.
    ///
    /// Raises:
    ///     RuntimeError: If the iteration fails.
    fn storage_iter<'py>(
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

            let results = api
                .storage()
                .at_latest()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .iter(storage_query)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok(StorageIterator {
                results: Arc::new(tokio::sync::Mutex::new(results)),
            })
        })
    }

    /// Sign and submit a transaction to the blockchain asynchronously.
    ///
    /// Args:
    ///     from (Keypair): The keypair to sign the transaction.
    ///     pallet_name (str): The name of the pallet.
    ///     entry_name (str): The name of the extrinsic.
    ///     payload (list): A list of arguments for the extrinsic.
    ///
    /// Returns:
    ///     str: The transaction hash as a hexadecimal string.
    ///
    /// Raises:
    ///     RuntimeError: If signing or submitting the transaction fails.
    fn sign_and_submit<'py>(
        &self,
        py: Python<'py>,
        from: Keypair,
        pallet_name: String,
        entry_name: String,
        payload: &PyList,
    ) -> PyResult<&'py PyAny> {
        let api = self.api.clone();
        let values: Vec<Value> = payload
            .iter()
            .map(|item| py_object_to_value(item, AddressUse::Extrinsic))
            .collect::<PyResult<Vec<Value>>>()?;
        future_into_py(py, async move {
            let tx_params = Params::new().build();
            let tx_payload = tx(pallet_name, entry_name, values);
            let hash = api
                .tx()
                .sign_and_submit(&tx_payload, &from, tx_params)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let hex_string = format!("{:?}", hash);
            Ok(hex_string)
        })
    }

    /// Subscribe to new blocks on the blockchain asynchronously.
    ///
    /// Returns:
    ///     BlockSubscription: An asynchronous iterator that yields blocks as they are finalized.
    ///
    /// Example:
    ///
    /// ```python
    /// async for block in client.subscribe_new_blocks():
    ///     print(block)
    /// ```
    ///
    /// Raises:
    ///     RuntimeError: If the subscription fails.
    fn subscribe_new_blocks<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let blocks = self.api.blocks();
        future_into_py(py, async move {
            let blocks_sub = blocks
                .subscribe_finalized()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok(BlockSubscription {
                blocks_stream: Arc::new(tokio::sync::Mutex::new(blocks_sub)),
            })
        })
    }
}

// Helper functions for converting values to Python objects

/// Convert a Python object to a Subxt `Value`.
///
/// Args:
///     item (PyAny): The Python object to convert.
///     address_use (AddressUse): Indicates whether the value is for storage or extrinsic use.
///
/// Returns:
///     Value: The corresponding Subxt value.
///
/// Raises:
///     TypeError: If the Python object type is unsupported.
fn py_object_to_value(item: &PyAny, address_use: AddressUse) -> PyResult<Value> {
    if let Ok(bytes) = item.downcast::<PyBytes>() {
        let bytes = bytes.as_bytes();
        Ok(Value::from_bytes(bytes.to_vec()))
    } else if let Ok(int_val) = item.extract::<i128>() {
        Ok(Value::i128(int_val))
    } else if let Ok(uint_val) = item.extract::<u128>() {
        Ok(Value::u128(uint_val))
    } else if let Ok(bool_val) = item.extract::<bool>() {
        Ok(Value::bool(bool_val))
    } else if let Ok(string_val) = item.downcast::<PyString>() {
        let s = string_val.to_string();
        if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            let bytes = hex::decode(&s).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid hex string: {}",
                    e
                ))
            })?;
            match address_use {
                AddressUse::Storage => Ok(Value::from_bytes(bytes)),
                AddressUse::Extrinsic => {
                    Ok(Value::unnamed_variant("Id", vec![Value::from_bytes(bytes)]))
                }
            }
        } else {
            Ok(Value::from_bytes(string_val.to_string().into_bytes()))
        }
    } else if let Ok(list_val) = item.downcast::<PyList>() {
        let values: PyResult<Vec<Value>> = list_val
            .iter()
            .map(|item| py_object_to_value(item, address_use.clone()))
            .collect();
        Ok(Value::unnamed_composite(values?))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported type in payload",
        ))
    }
}

/// Convert a Composite value to a Python object.
///
/// Args:
///     py (Python): The Python interpreter state.
///     composite (Composite<u32>): The composite value to convert.
///
/// Returns:
///     PyObject: The corresponding Python object.
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

/// Convert a Primitive value to a Python object.
///
/// Args:
///     py (Python): The Python interpreter state.
///     primitive (Primitive): The primitive value to convert.
///
/// Returns:
///     PyObject: The corresponding Python object.
fn primitive_to_py_object(py: Python, primitive: &Primitive) -> PyResult<PyObject> {
    match primitive {
        Primitive::Bool(b) => Ok(b.to_object(py)),
        Primitive::Char(c) => Ok(c.to_object(py)),
        Primitive::String(s) => Ok(s.to_object(py)),
        Primitive::U128(u) => Ok(u.to_object(py)),
        Primitive::I128(i) => Ok(i.to_object(py)),
        _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported primitive type",
        )),
    }
}

/// Convert a decoded value to a Python object.
///
/// Args:
///     py (Python): The Python interpreter state.
///     decoded_value (Value<u32>): The decoded value to convert.
///
/// Returns:
///     PyObject: The corresponding Python object.
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
                    let py_values = PyList::new(
                        py,
                        unnamed
                            .iter()
                            .map(|v| decoded_value_to_py_object(py, v).unwrap()),
                    );
                    py_dict.set_item("values", py_values)?;
                }
            }

            Ok(py_dict.into())
        }
        ValueDef::BitSequence(bit_sequence) => {
            let bits: Vec<bool> = bit_sequence.iter().collect();
            Ok(PyList::new(py, bits).into())
        }
        ValueDef::Primitive(primitive) => primitive_to_py_object(py, primitive),
    }
}

/// Define the Python module.
#[pymodule]
fn subxtpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SubxtClient>()?;
    m.add_class::<StorageIterator>()?;
    m.add_class::<Keypair>()?;
    m.add_class::<BlockSubscription>()?;
    Ok(())
}
