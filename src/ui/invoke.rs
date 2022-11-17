use crate::stream::{Contract, Event, EventBody};
use crate::vm::invoke::invoke;

use soroban_env_host::events::HostEvent;
use soroban_env_host::{budget::Budget, events::Events, xdr::ReadXdr};
use stellar_xdr::WriteXdr;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::NodeRef;
use yew::{
    events,
    prelude::{html, Component, Context, Html},
    Properties, TargetCast,
};

#[derive(Default)]
pub struct InvokeComp {
    function: Option<String>,
    result: Option<String>,
    budget: Option<Budget>,
    events: Option<Events>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct InvokeCompProps {
    pub contract: Contract,
    pub event: Event,
    pub related_events: Vec<Event>,
}

pub enum InvokeCompMsg {
    SelectFunction { function: String },
    Invoke { args: String },
}

impl Component for InvokeComp {
    type Message = InvokeCompMsg;
    type Properties = InvokeCompProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InvokeCompMsg::SelectFunction { function } => {
                self.function = Some(function);
                false
            }
            InvokeCompMsg::Invoke { args } => {
                if let Some(function) = &self.function {
                    let props = ctx.props();
                    let contract = &props.contract;
                    let mut related_events = props.related_events.clone();
                    related_events.sort_by(|a, b| a.id.cmp(&b.id));
                    let related_invocations = related_events
                        .iter()
                        .filter_map(|e| {
                            if let EventBody::Invocation(i) = &e.body {
                                Some(i.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let mut storage = None;
                    for i in related_invocations {
                        let (_, new_storage, _, _) = invoke(
                            storage,
                            contract.bytes.clone(),
                            contract.id.clone(),
                            function.clone(),
                            i.args
                                .iter()
                                .map(|a| match a {
                                    Some(a) => a.clone(),
                                    None => stellar_xdr::ScVal::Static(stellar_xdr::ScStatic::Void),
                                })
                                .map(|a| {
                                    soroban_env_host::xdr::ScVal::from_xdr_base64(
                                        a.to_xdr_base64().unwrap(),
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        );
                        storage = Some(new_storage);
                    }
                    let (result, _, budget, events) = invoke(
                        storage,
                        contract.bytes.clone(),
                        contract.id.clone(),
                        function.clone(),
                        serde_json::from_str(&args).unwrap(),
                    );
                    self.result = Some(result);
                    self.budget = Some(budget);
                    self.events = Some(events);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.function = None;
        self.result = None;
        self.budget = None;
        self.events = None;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = ctx.link();
        let props = ctx.props();
        let contract = &props.contract;
        let functions = contract.fns();
        let events = if let Some(Events(events)) = &self.events {
            events
                .iter()
                .map(|e| match e {
                    HostEvent::Contract(e) => serde_json::to_string_pretty(e).unwrap_or_default(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let budget = if let Some(budget) = &self.budget {
            format!(
                "cpu: {} mem: {}",
                budget.get_cpu_insns_count(),
                budget.get_mem_bytes_count()
            )
        } else {
            String::new()
        };
        let onchange = {
            scope.callback(|e: events::Event| InvokeCompMsg::SelectFunction {
                function: e.target_unchecked_into::<HtmlSelectElement>().value(),
            })
        };
        let args_ref = NodeRef::default();
        let args_ref_in_html = args_ref.clone();
        let onclick = {
            ctx.link().callback(move |_| InvokeCompMsg::Invoke {
                args: {
                    args_ref
                        .cast::<HtmlInputElement>()
                        .map(|n| n.value())
                        .unwrap_or_default()
                },
            })
        };
        html! {
            <div class="component invoke">
                <strong>{ "function: " }</strong>
                <select {onchange}>
                    <option value="">{ "-- select a function --" }</option>
                    {
                        for functions.iter().map(|f| {
                            html! { <option value={f.clone()}>{f}</option> }
                        })
                    }
                </select>
                <button {onclick}>{ "invoke" }</button>
                <br/>
                <strong>{ "args: " }</strong>{ " (json array of scvals)"}
                <br/>
                <textarea ref={args_ref_in_html} value="[]" />
                <br/>
                <hr/>
                <strong>{ "result: " }</strong>
                <br/>
                <pre><code>{ self.result.clone().unwrap_or_default() }</code></pre>
                <br/>
                <strong>{ "events: " }</strong>
                <br/>
                <pre><code>{ events }</code></pre>
                <br/>
                <strong>{ "budget: " }</strong>
                <br/>
                <pre><code>{ budget }</code></pre>
            </div>
        }
    }
}
