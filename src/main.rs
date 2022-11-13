mod horizon;

use std::rc::Rc;

use soroban_spec::gen::rust::ToFormattedString;
use stellar_xdr::{
    InvokeHostFunctionResult, OperationResult, OperationResultTr, ReadXdr, ScObject, ScSpecEntry,
    ScSpecFunctionV0, ScVal, TransactionResult, TransactionResultResult,
};
use yew::{
    prelude::{html, Component, Context, Html},
    start_app,
};

const HORIZON_BASE_URL: &str = "https://horizon-futurenet.stellar.org";

fn main() {
    start_app::<App>();
}

#[derive(Default)]
struct App {
    events: Vec<Event>,
    display: Option<String>,
    invoke_result: Option<String>,
}

struct Event {
    tx: String,
    at: String,
    body: EventBody,
}

enum EventBody {
    Deployment(Contract),
}

struct Contract {
    id: String,
    bytes: Vec<u8>,
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

enum Msg {
    Event(Event),
    Display(String),
    Invoke(Vec<u8>, String, String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async {
            collect(Order::Desc, move |message| {
                link.send_message(message);
            })
            .await;
        });
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Event(e) => {
                self.events.push(e);
                true
            }
            Msg::Display(h) => {
                if self.display.as_deref() == Some(&h) {
                    false
                } else {
                    self.display = Some(h);
                    self.invoke_result = None;
                    true
                }
            }
            Msg::Invoke(code, id, function) => {
                self.invoke_result = Some(invoke(code, id, function));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="columns">
                <div class="left">
                <table>
                <tr><th>{ "at" }</th><th>{ "tx" }</th><th>{ "op" }</th><th>{ "hash" }</th><th>{ "id" }</th></tr>
                {
                    for self.events.iter().map(|e| {
                        let tx_hash = e.tx.clone();
                        let tx_url = format!("{}/transactions/{}", HORIZON_BASE_URL, tx_hash);
                        let selected = self.display.as_deref() == Some(&tx_hash);
                        match &e.body {
                            EventBody::Deployment(c) => {
                                let c_id = c.id.clone();
                                let c_hash = c.hash();
                                html!{
                                    <tr class={ if selected { "selected" } else { "" } }>
                                        <td>{ &e.at }</td>
                                        <td><a href={ tx_url } target="_blank">{ &e.tx[..7] }</a></td>
                                        <td>{ "deploy" }</td>
                                        <td>{ &c_hash[..7] }</td>
                                        <td>{ &c_id[..7] }</td>
                                        <td><button onclick={ctx.link().callback(move |_| Msg::Display(tx_hash.clone()))}>{ "view" }</button></td>
                                    </tr>
                                }
                            },
                        }
                    })
                }
                </table>
                </div>
                <div class="right">
                {
                    for self.events.iter().rev().filter_map(|e| {
                        let tx_hash = e.tx.clone();
                        let tx_url = format!("{}/transactions/{}", HORIZON_BASE_URL, tx_hash);
                        let selected = self.display.as_deref() == Some(&tx_hash);
                        if !selected {
                            return None;
                        }
                        match &e.body {
                            EventBody::Deployment(c) => {
                                let c_id = c.id.clone();
                                let c_hash = c.hash();
                                let c_bytes = c.bytes.clone();
                                let download_href = format!("data:application/pdf;base64,{}", base64::encode(&c.bytes));
                                let download_filename = format!("{}.wasm", &c_hash[..7]);
                                let c_fn_first = c.fns()[0].clone();
                                let invoke_result = self.invoke_result.clone();
                                Some(html!{
                                    <div class="expand">
                                        <div class="box">
                                            <strong>{ "tx: " }</strong><a href={ tx_url } target="_blank">{ &e.tx }</a><br/>
                                            <strong>{ "at: " }</strong>{ &e.at }<br/>
                                            <strong>{ "event: " }</strong>{ "deploy" }<br/>
                                            <strong>{ "contract hash: " }</strong>{ &c_hash }<br/>
                                            <strong>{ "contract id: " }</strong>{ &c_id }<br/>
                                            <strong>{ "download: " }</strong><a href={download_href} target="_self" download={download_filename.clone()}>{download_filename}</a><br/>
                                            <code>{ c.spec_html() }</code>
                                        </div>
                                        <div class="box">
                                            <strong>{ "function: " }</strong>
                                            <select>
                                            {
                                                for c.fns().iter().map(|f| {
                                                    html! { <option value={f.clone()}>{f}</option> }
                                                })
                                            }
                                            </select>
                                            <td><button onclick={ctx.link().callback(move |_| Msg::Invoke(c_bytes.clone(), c_id.clone(), c_fn_first.clone()))}>{ "invoke" }</button></td>
                                            <br/>
                                            <strong>{ "result: " }</strong>
                                            <br/>
                                            <code>{ invoke_result.unwrap_or_default() }</code>
                                        </div>
                                    </div>
                                })
                            },
                        }
                    })
                }
                </div>
            </div>
        }
    }
}

enum Order {
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

async fn collect(o: Order, f: impl Fn(Msg)) {
    let mut next = format!(
        "{}/operations?order={}",
        HORIZON_BASE_URL,
        o.query_param_value()
    );
    loop {
        let backoff = backoff::ExponentialBackoff::default();
        let resp = backoff::future::retry(backoff, || async {
            let result = reqwest::get(&next).await;
            match result {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<horizon::operations::Response>().await {
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
                let tx = get_transaction(&r.transaction_hash).await;
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
                    f(Msg::Event(Event {
                        tx: r.transaction_hash.clone(),
                        at: r.created_at.clone(),
                        body: EventBody::Deployment(Contract { id, bytes }),
                    }));
                }
            }
        }
    }
}

async fn get_transaction(hash: &str) -> horizon::transaction::Response {
    let url = format!("{HORIZON_BASE_URL}/transactions/{hash}");
    let backoff = backoff::ExponentialBackoff::default();
    backoff::future::retry(backoff, || async {
        let result = reqwest::get(&url).await;
        match result {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<horizon::transaction::Response>().await {
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

fn invoke(code: Vec<u8>, id: String, function: String) -> String {
    use soroban_env_host::{
        budget::Budget,
        storage::{SnapshotSource, Storage},
        xdr::{
            self, ContractDataEntry, Hash, HostFunction, LedgerEntry, LedgerEntryData,
            LedgerEntryExt, LedgerKey, LedgerKeyContractData, ScContractCode,
            ScHostStorageErrorCode, ScObject, ScStatic, ScStatus, ScVal,
        },
        Host, HostError, Status,
    };
    struct CodeOnlySnapshotSource(Hash, ScContractCode);
    impl CodeOnlySnapshotSource {
        fn key(&self) -> LedgerKey {
            LedgerKey::ContractData(LedgerKeyContractData {
                contract_id: self.0.clone(),
                key: ScVal::Static(ScStatic::LedgerKeyContractCode),
            })
        }
        fn data(&self) -> LedgerEntryData {
            LedgerEntryData::ContractData(ContractDataEntry {
                contract_id: self.0.clone(),
                key: ScVal::Static(ScStatic::LedgerKeyContractCode),
                val: ScVal::Object(Some(ScObject::ContractCode(self.1.clone()))),
            })
        }
    }
    impl SnapshotSource for CodeOnlySnapshotSource {
        fn get(&self, key: &xdr::LedgerKey) -> Result<xdr::LedgerEntry, HostError> {
            if key == &self.key() {
                Ok(LedgerEntry {
                    last_modified_ledger_seq: 0,
                    data: self.data(),
                    ext: LedgerEntryExt::V0,
                })
            } else {
                let status: Status =
                    ScStatus::HostStorageError(ScHostStorageErrorCode::UnknownError).into();
                Err(status.into())
            }
        }
        fn has(&self, key: &xdr::LedgerKey) -> Result<bool, HostError> {
            Ok(key == &self.key())
        }
    }
    let hex_id = hex::decode(&id).unwrap();
    let storage = Storage::with_recording_footprint(Rc::new(CodeOnlySnapshotSource(
        (&hex_id).try_into().unwrap(),
        ScContractCode::Wasm(code.try_into().unwrap()),
    )));
    let h = Host::with_storage_and_budget(storage, Budget::default());
    let res = h
        .invoke_function(
            HostFunction::InvokeContract,
            vec![
                ScVal::Object(Some(ScObject::Bytes(hex_id.try_into().unwrap()))),
                ScVal::Symbol((&function).try_into().unwrap()),
                ScVal::Symbol("asdf".try_into().unwrap()),
            ]
            .try_into()
            .unwrap(),
        )
        .unwrap();
    serde_json::to_string_pretty(&res).unwrap()

    // let (storage, budget, events) = h.try_finish().map_err(|_h| {
    //     HostError::from(ScStatus::HostStorageError(
    //         ScHostStorageErrorCode::UnknownError,
    //     ))
    // })?;

    //     eprintln!(
    //         "Footprint: {}",
    //         serde_json::to_string(&create_ledger_footprint(&storage.footprint)).unwrap(),
    //     );

    // if self.cost {
    //     eprintln!("Cpu Insns: {}", budget.get_cpu_insns_count());
    //     eprintln!("Mem Bytes: {}", budget.get_mem_bytes_count());
    //     for cost_type in CostType::variants() {
    //         eprintln!("Cost ({cost_type:?}): {}", budget.get_input(*cost_type));
    //     }
    // }

    // for (i, event) in events.0.iter().enumerate() {
    //     eprint!("#{i}: ");
    //     match event {
    //         HostEvent::Contract(e) => {
    //             eprintln!("event: {}", serde_json::to_string(&e).unwrap());
    //         }
    //         HostEvent::Debug(e) => eprintln!("debug: {e}"),
    //     }
    // }
}
