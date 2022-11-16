use super::horizonapi;
use soroban_spec::gen::rust::ToFormattedString;
use stellar_xdr::{
    InvokeHostFunctionResult, OperationResult, OperationResultTr, ReadXdr, ScObject, ScSpecEntry,
    ScSpecFunctionV0, ScVal, TransactionResult, TransactionResultResult,
};
use yew::prelude::{html, Html};

pub struct Event {
    pub tx: String,
    pub at: String,
    pub body: EventBody,
}

pub enum EventBody {
    Deployment(Contract),
}

pub struct Contract {
    pub id: String,
    pub bytes: Vec<u8>,
}

impl Contract {
    pub fn hash(&self) -> String {
        sha256::digest(self.bytes.as_slice())
    }

    pub fn fns(&self) -> Vec<String> {
        soroban_spec::read::from_wasm(&self.bytes)
            .unwrap()
            .into_iter()
            .filter_map(|s| match s {
                ScSpecEntry::FunctionV0(ScSpecFunctionV0 { name, .. }) => {
                    Some(name.to_string_lossy())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn spec_rust(&self) -> String {
        let rust = soroban_spec::gen::rust::generate_from_wasm(
            self.bytes.as_slice(),
            "contract.wasm",
            None,
        )
        .unwrap();
        rust.to_formatted_string().unwrap()
    }

    pub fn spec_html(&self) -> Html {
        let rust = self.spec_rust().replace("soroban_sdk::", "");
        html!(<code>{ rust }</code>)
    }
}

pub enum Order {
    Asc,
    Desc,
}

impl Order {
    fn query_param_value(&self) -> &'static str {
        match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        }
    }
}

pub async fn collect(base_url: &str, o: Order, f: impl Fn(Event)) {
    let mut next = format!("{base_url}/operations?order={}", o.query_param_value());
    loop {
        let backoff = backoff::ExponentialBackoff::default();
        let resp = backoff::future::retry(backoff, || async {
            let result = reqwest::get(&next).await;
            match result {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<horizonapi::operations::Response>().await {
                            Ok(resp) => Ok(resp),
                            Err(_) => Err(backoff::Error::transient(())),
                        }
                    } else {
                        Err(backoff::Error::transient(()))
                    }
                }
                Err(_) => Err(backoff::Error::transient(())),
            }
        })
        .await
        .unwrap();

        let next_next = resp.links.next.href;
        if next_next == next {
            break;
        }
        next = next_next;

        let records = resp
            .embedded
            .records
            .iter()
            .filter(|r| r.r#type == "invoke_host_function");

        for r in records {
            if r.function.as_deref() == Some("HostFunctionHostFnCreateContractWithSourceAccount") {
                let tx = get_transaction(base_url, &r.transaction_hash).await;
                let id = if let Ok(TransactionResult {
                    result: TransactionResultResult::TxSuccess(op_results),
                    ..
                }) = TransactionResult::from_xdr_base64(tx.result_xdr)
                {
                    if let Some(OperationResult::OpInner(OperationResultTr::InvokeHostFunction(
                        InvokeHostFunctionResult::Success(ScVal::Object(Some(ScObject::Bytes(id)))),
                    ))) = op_results.get(0)
                    {
                        Some(hex::encode(id))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let bytes = if let Some(code) = r.parameters.get(0) {
                    if let Ok(ScVal::Object(Some(ScObject::Bytes(bytes)))) =
                        ScVal::from_xdr_base64(&code.value)
                    {
                        Some(bytes.into())
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let (Some(id), Some(bytes)) = (id, bytes) {
                    f(Event {
                        tx: r.transaction_hash.clone(),
                        at: r.created_at.clone(),
                        body: EventBody::Deployment(Contract { id, bytes }),
                    });
                }
            }
        }
    }
}

pub async fn get_transaction(base_url: &str, hash: &str) -> horizonapi::transaction::Response {
    let url = format!("{base_url}/transactions/{hash}");
    let backoff = backoff::ExponentialBackoff::default();
    backoff::future::retry(backoff, || async {
        let result = reqwest::get(&url).await;
        match result {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<horizonapi::transaction::Response>().await {
                        Ok(resp) => Ok(resp),
                        Err(_) => Err(backoff::Error::transient(())),
                    }
                } else {
                    Err(backoff::Error::transient(()))
                }
            }
            Err(_) => Err(backoff::Error::transient(())),
        }
    })
    .await
    .unwrap()
}
