use crate::stream::Contract;
use crate::vm::invoke::invoke;

use web_sys::HtmlSelectElement;
use yew::{
    events,
    prelude::{html, Component, Context, Html},
    Properties, TargetCast,
};

#[derive(Default)]
pub struct InvokeComp {
    function: Option<String>,
    result: Option<String>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct InvokeCompProps {
    pub contract: Contract,
}

pub enum InvokeCompMsg {
    SelectFunction { function: String },
    Invoke,
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
            InvokeCompMsg::Invoke => {
                let props = ctx.props();
                let contract = &props.contract;
                self.result = Some(invoke(
                    contract.bytes.clone(),
                    contract.id.clone(),
                    self.function.clone().unwrap(),
                ));
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.function = None;
        self.result = None;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = ctx.link();
        let props = ctx.props();
        let contract = &props.contract;
        let functions = contract.fns();
        let onchange = {
            scope.callback(|e: events::Event| InvokeCompMsg::SelectFunction {
                function: e.target_unchecked_into::<HtmlSelectElement>().value(),
            })
        };
        let onclick = { ctx.link().callback(|_| InvokeCompMsg::Invoke) };
        html! {
            <div class="component invoke">
                <strong>{ "function: " }</strong>
                <select {onchange}>
                    <option disabled=true selected=true value="">{ "-- select a function --" }</option>
                    {
                        for functions.iter().map(|f| {
                            html! { <option value={f.clone()}>{f}</option> }
                        })
                    }
                </select>
                <button {onclick}>{ "invoke" }</button>
                <br/>
                <strong>{ "result: " }</strong>
                <br/>
                <code>{ self.result.clone().unwrap_or_default() }</code>
            </div>
        }
    }
}
