use crate::stream::Invocation;

use yew::{
    prelude::{html, Component, Context, Html},
    Properties,
};

#[derive(Default)]
pub struct InvokeInfoComp;

#[derive(Clone, PartialEq, Properties)]
pub struct InvokeInfoCompProps {
    pub invocation: Invocation,
}

impl Component for InvokeInfoComp {
    type Message = ();
    type Properties = InvokeInfoCompProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let invocation = &props.invocation;
        html! {
            <div class="component invocationinfo">
                <strong>{ "contract id: " }</strong>{ &invocation.id }<br/>
                <strong>{ "function: " }</strong>{ &invocation.function }<br/>
                <strong>{ "args: " }</strong><br/>
                <pre><code class="language-json">{ serde_json::to_string_pretty(&invocation.args).unwrap_or_default() }</code></pre>
                <strong>{ "result: " }</strong><br/>
                <pre><code class="language-json">{ serde_json::to_string_pretty(&invocation.result).unwrap_or_default() }</code></pre>
                <strong>{ "events: " }</strong><br/>
                <pre><code class="language-json">{ serde_json::to_string_pretty(&invocation.events).unwrap_or_default() }</code></pre>
                <strong>{ "footprint: " }</strong><br/>
                <pre><code class="language-json">{ serde_json::to_string_pretty(&invocation.footprint).unwrap_or_default() }</code></pre>
            </div>
        }
    }
}
