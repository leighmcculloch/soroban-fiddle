use crate::{
    stream::{collect, Event, EventBody, Order},
    HORIZON_BASE_URL,
};

use yew::{
    prelude::{html, Component, Context, Html},
    Callback, Properties,
};

#[derive(Default)]
pub struct HistoryComp {
    events: Vec<Event>,
    selected_event: Option<Event>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct HistoryCompProps {
    pub onevent: Callback<Event>,
}

pub enum HistoryCompMsg {
    Event(Event),
    SelectEvent(Event),
}

impl Component for HistoryComp {
    type Message = HistoryCompMsg;
    type Properties = HistoryCompProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async {
            collect(HORIZON_BASE_URL, Order::Desc, move |event| {
                link.send_message(HistoryCompMsg::Event(event));
            })
            .await;
        });
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HistoryCompMsg::Event(e) => {
                self.events.push(e);
                true
            }
            HistoryCompMsg::SelectEvent(e) => {
                if self.selected_event.as_ref() == Some(&e) {
                    false
                } else {
                    self.selected_event = Some(e.clone());
                    ctx.props().onevent.emit(e);
                    true
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = ctx.link();
        let events = self.events.clone();
        html! {
            <div class="component history">
                <table>
                <tr><th>{ "at" }</th><th>{ "tx" }</th><th>{ "op" }</th><th>{ "hash" }</th><th>{ "id" }</th></tr>
                {
                    for events.into_iter().map(|e| {
                        let tx_hash = e.tx.clone();
                        let tx_url = format!("{}/transactions/{}", HORIZON_BASE_URL, tx_hash);
                        let selected = self.selected_event.as_ref().map(|e| &e.tx) == Some(&tx_hash);
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
                                        <td><button onclick={scope.callback(move |_| HistoryCompMsg::SelectEvent(e.clone()))}>{ "view" }</button></td>
                                    </tr>
                                }
                            },
                        }
                    })
                }
                </table>
            </div>
        }
    }
}
