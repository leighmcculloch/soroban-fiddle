use std::{fmt::Display, str::FromStr};

use crate::stream::Contract;

use web_sys::HtmlSelectElement;
use yew::{
    events,
    prelude::{html, Component, Context, Html},
    Properties, TargetCast,
};

#[derive(Default)]
pub struct ContractInfoComp {
    format: Format,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ContractInfoCompProps {
    pub contract: Contract,
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Format {
    Rust,
    Json,
}

impl Format {
    pub fn all() -> &'static [Format] {
        &[Format::Rust, Format::Json]
    }
}

impl FromStr for Format {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(Format::Rust),
            "json" => Ok(Format::Json),
            _ => Err(()),
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Rust => write!(f, "rust"),
            Format::Json => write!(f, "json"),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Rust
    }
}

pub enum ContractInfoCompMsg {
    SelectFormat { format: Format },
}

impl Component for ContractInfoComp {
    type Message = ContractInfoCompMsg;
    type Properties = ContractInfoCompProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ContractInfoCompMsg::SelectFormat { format } => {
                self.format = format;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = ctx.link();
        let props = ctx.props();
        let contract = &props.contract;
        let download_href = format!(
            "data:application/pdf;base64,{}",
            base64::encode(&contract.bytes)
        );
        let download_filename = format!("{}.wasm", &contract.hash()[..7]);
        let onchange = {
            scope.callback(|e: events::Event| ContractInfoCompMsg::SelectFormat {
                format: Format::from_str(&e.target_unchecked_into::<HtmlSelectElement>().value())
                    .unwrap_or_default(),
            })
        };

        html! {
            <div class="component contractinfo">
                <strong>{ "contract hash: " }</strong>{ &contract.hash() }<br/>
                <strong>{ "contract id: " }</strong>{ &contract.id }<br/>
                <strong>{ "download: " }</strong><a href={download_href} target="_self" download={download_filename.clone()}>{download_filename}</a><br/>
                <select {onchange}>
                    {
                        for Format::all().iter().map(|f| {
                            html! { <option value={f.to_string()} selected={f == &self.format}>{f}</option> }
                        })
                    }
                </select><br/>
                {
                    match self.format {
                        Format::Rust => html! {
                            <pre><code class="language-rust">{ props.contract.spec_rust() }</code></pre>
                        },
                        Format::Json => html! {
                            <pre><code class="language-json">{ props.contract.spec_json() }</code></pre>
                        },
                    }
                }
            </div>
        }
    }
}
