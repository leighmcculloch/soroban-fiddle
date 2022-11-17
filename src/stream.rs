use std::time::Duration;

use super::horizonapi;
use soroban_spec::gen::rust::ToFormattedString;
use stellar_xdr::{
    ContractEvent, InvokeHostFunctionResult, LedgerFootprint, OperationResult, OperationResultTr,
    ReadXdr, ScObject, ScSpecEntry, ScSpecFunctionV0, ScVal, TransactionMeta, TransactionMetaV3,
    TransactionResult, TransactionResultResult,
};

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Event {
    pub id: String,
    pub tx: String,
    pub at: String,
    pub body: EventBody,
}

impl Event {
    pub fn contract_id(&self) -> String {
        match &self.body {
            EventBody::Invocation(i) => i.id.clone(),
            EventBody::Deployment(d) => d.id.clone(),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum EventBody {
    Invocation(Invocation),
    Deployment(Contract),
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Invocation {
    pub id: String,
    pub function: String,
    pub args: Vec<Option<ScVal>>,
    pub result: Option<ScVal>,
    pub footprint: Option<LedgerFootprint>,
    pub events: Option<Vec<ContractEvent>>,
}

#[derive(Clone, PartialEq, PartialOrd)]
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
        soroban_spec::gen::rust::generate_from_wasm(self.bytes.as_slice(), "contract.wasm", None)
            .unwrap()
            .to_formatted_string()
            .unwrap()
            .replace("soroban_sdk::", "")
    }

    pub fn spec_json(&self) -> String {
        soroban_spec::gen::json::generate_from_wasm(self.bytes.as_slice()).unwrap()
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

pub async fn latest_event_and_cursor(base_url: &str) -> (Option<Event>, Option<String>) {
    let url = format!("{base_url}/operations?order=desc&limit=1");
    let (events, cursor, _) = get_operations(base_url, &url).await;
    (events.first().cloned(), cursor)
}

pub async fn collect_events(
    base_url: &str,
    cursor: &str,
    o: Order,
    d: Duration,
    f: impl Fn(Event),
) {
    let mut next = get_operations_url(base_url, cursor, o, 10);
    loop {
        let (events, _, next_url) = get_operations(base_url, &next).await;
        for e in events {
            f(e);
        }
        next = next_url;
        gloo_timers::future::sleep(d).await;
    }
}

pub fn get_operations_url(base_url: &str, cursor: &str, o: Order, limit: usize) -> String {
    format!(
        "{base_url}/operations?cursor={}&order={}&limit={}",
        cursor,
        o.query_param_value(),
        limit,
    )
}

pub async fn get_operations(base_url: &str, url: &str) -> (Vec<Event>, Option<String>, String) {
    let backoff = backoff::ExponentialBackoff::default();
    let resp = backoff::future::retry(backoff, || async {
        let result = reqwest::get(url).await;
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

    let records = resp
        .embedded
        .records
        .iter()
        .filter(|r| r.r#type == "invoke_host_function");

    let mut events: Vec<Event> = vec![];
    for r in records {
        match r.function.as_deref() {
            Some("HostFunctionHostFnInvokeContract") => {
                let id = if let Some(id) = r.parameters.get(0) {
                    if let Ok(ScVal::Object(Some(ScObject::Bytes(id)))) =
                        ScVal::from_xdr_base64(&id.value)
                    {
                        Some(hex::encode(id))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let function = if let Some(function) = r.parameters.get(1) {
                    if let Ok(ScVal::Symbol(function)) = ScVal::from_xdr_base64(&function.value) {
                        Some(function.to_string_lossy())
                    } else {
                        None
                    }
                } else {
                    None
                };
                let args = r
                    .parameters
                    .iter()
                    .skip(2)
                    .map(|a| ScVal::from_xdr_base64(&a.value).ok())
                    .collect::<Vec<_>>();
                let tx = get_transaction(base_url, &r.transaction_hash).await;
                let result = if let Ok(TransactionResult {
                    result: TransactionResultResult::TxSuccess(op_results),
                    ..
                }) = TransactionResult::from_xdr_base64(tx.result_xdr)
                {
                    if let Some(OperationResult::OpInner(OperationResultTr::InvokeHostFunction(
                        InvokeHostFunctionResult::Success(result),
                    ))) = op_results.get(0)
                    {
                        Some(result.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                let contract_events =
                    if let Ok(TransactionMeta::V3(TransactionMetaV3 { events, .. })) =
                        TransactionMeta::from_xdr_base64(tx.result_meta_xdr)
                    {
                        Some(events.into())
                    } else {
                        None
                    };
                let footprint = if let Some(footprint) = &r.footprint {
                    if let Ok(footprint) = LedgerFootprint::from_xdr_base64(footprint) {
                        Some(footprint)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let (Some(id), Some(function)) = (id, function) {
                    events.push(Event {
                        id: r.id.clone(),
                        tx: r.transaction_hash.clone(),
                        at: r.created_at.clone(),
                        body: EventBody::Invocation(Invocation {
                            id,
                            function,
                            args,
                            result,
                            footprint,
                            events: contract_events,
                        }),
                    });
                }
            }
            Some("HostFunctionHostFnCreateContractWithSourceAccount") => {
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
                    events.push(Event {
                        id: r.id.clone(),
                        tx: r.transaction_hash.clone(),
                        at: r.created_at.clone(),
                        body: EventBody::Deployment(Contract { id, bytes }),
                    });
                }
            }
            _ => {}
        }
    }
    (
        events,
        resp.embedded
            .records
            .first()
            .map(|r| r.paging_token.clone()),
        resp.links.next.href,
    )
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
