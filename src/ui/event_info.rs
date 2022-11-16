use crate::{
    stream::{Event, EventBody},
    HORIZON_BASE_URL,
};

use yew::{
    prelude::{html, Component, Context, Html},
    Properties,
};

#[derive(Default)]
pub struct EventInfoComp {}

#[derive(Clone, PartialEq, Properties)]
pub struct EventInfoCompProps {
    pub event: Event,
}

pub enum EventInfoCompMsg {}

impl Component for EventInfoComp {
    type Message = EventInfoCompMsg;
    type Properties = EventInfoCompProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let event = &props.event;
        let tx_url = format!("{}/transactions/{}", HORIZON_BASE_URL, event.tx);
        html! {
            <div class="component eventinfo">
                <strong>{ "tx: " }</strong><a href={ tx_url } target="_blank">{ &event.tx }</a><br/>
                <strong>{ "at: " }</strong>{ &event.at }<br/>
                <strong>{ "event: " }</strong>
                {
                    match &event.body {
                        EventBody::Invocation(_) => "invoke",
                        EventBody::Deployment(_) => "deploy",
                    }
                }
            </div>
        }
    }
}
