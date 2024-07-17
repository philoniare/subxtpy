mod storage_fetch;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use subxt::{OnlineClient as SOnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;
use tokio::runtime::Runtime;
use subxt::dynamic::{At, Value};

#[pyclass]
struct OnlineClient {
    client: Option<SOnlineClient<PolkadotConfig>>,
}

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod polkadot {}

#[pymethods]
impl OnlineClient {
    #[new]
    pub fn new(py: Python) -> Self {
        Self { client: None }
    }

    fn initialize(&mut self, py: Python) -> PyResult<()> {
        py.allow_threads(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let client = SOnlineClient::<PolkadotConfig>::new().await.unwrap();
                self.client = Some(client);
            });
        });
        Ok(())
    }

    fn constants(&self) -> PyResult<()> {
        let constant_query = subxt::dynamic::constant("System", "BlockHashCount");
        let value = self
            .client
            .as_ref()
            .unwrap()
            .constants()
            .at(&constant_query)
            .unwrap();
        println!("value: {:?}", value.encoded());
        Ok(())
    }

    #[getter]
    fn block_length(&self) -> PyResult<String> {
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async move {
            let constant_query = polkadot::constants().system().block_length();
            let value = self
                .client
                .as_ref()
                .unwrap()
                .constants()
                .at(&constant_query)
                .unwrap();

            Ok(format!("{value:?}"))
        });
        result
    }

    fn storagy_query(&self, py: Python) -> PyResult<()> {
        let account = dev::alice().public_key();
        let storage_query =
            subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(account)]);

        py.allow_threads(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let result = self.client.as_ref().unwrap()
                    .storage()
                    .at_latest()
                    .await.unwrap()
                    .fetch(&storage_query)
                    .await.unwrap();
                let value = result.unwrap().to_value().unwrap();

                println!("Alice has free balance: {:?}", value.at("data").at("free"));
            });
        });
        Ok(())
    }
}

// fn register_polkadot_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
//     let child_module = PyModule::new_bound(parent_module.py(), "child_module")?;
//     child_module.add_function(wrap_pyfunction!(block_length, &child_module)?)?;
//     parent_module.add_submodule(&child_module)?;
//     Ok(())
// }

#[pymodule]
fn subxtpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // register_polkadot_module(m)?;
    m.add_class::<OnlineClient>()?;
    // m.add_wrapped(wrap_pyfunction!(encoded))?;
    // m.add_wrapped(wrap_pyfunction!(to_value))?;
    Ok(())
}
