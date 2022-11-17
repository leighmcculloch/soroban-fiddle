use std::time::Duration;

use crate::{
    stream::{collect_events, latest_event_and_cursor, Event, EventBody, Order},
    HORIZON_BASE_URL,
};

use yew::{
    prelude::{html, Component, Context, Html},
    Callback, Properties,
};

use futures::future::join;

#[derive(Default)]
pub struct HistoryComp {
    events: Vec<Event>,
    selected_event: Option<Event>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct HistoryCompProps {
    pub onevent: Callback<SelectedEvent>,
}

pub struct SelectedEvent {
    pub event: Event,
    pub related: Vec<Event>,
}

pub enum HistoryCompMsg {
    Event(Event),
    SelectEvent(Event),
}

impl Component for HistoryComp {
    type Message = HistoryCompMsg;
    type Properties = HistoryCompProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link_asc = ctx.link().clone();
        let link_desc = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async {
            let (event, cursor) = latest_event_and_cursor(HORIZON_BASE_URL).await;
            if let Some(event) = event {
                link_asc.send_message(HistoryCompMsg::Event(event));
            }
            if let Some(cursor) = cursor {
                join(
                    collect_events(HORIZON_BASE_URL, &cursor, Order::Asc, Duration::from_secs(3), move |event| {
                        link_asc.send_message(HistoryCompMsg::Event(event));
                    }),
                    collect_events(HORIZON_BASE_URL, &cursor, Order::Desc, Duration::from_secs(1), move |event| {
                        link_desc.send_message(HistoryCompMsg::Event(event));
                    }),
                )
                .await;
            }
        });
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HistoryCompMsg::Event(e) => {
                let found = self.events.binary_search_by(|f| f.id.cmp(&e.id).reverse());
                let i = match found {
                    Ok(i) | Err(i) => i,
                };
                self.events.insert(i, e);
                true
            }
            HistoryCompMsg::SelectEvent(e) => {
                if self.selected_event.as_ref() == Some(&e) {
                    false
                } else {
                    self.selected_event = Some(e.clone());
                    let related = self
                        .events
                        .iter()
                        .filter(|r| r.contract_id() == e.contract_id())
                        .cloned()
                        .collect();
                    ctx.props()
                        .onevent
                        .emit(SelectedEvent { event: e, related });
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
                            EventBody::Invocation(i) => {
                                let c_id = i.id.clone();
                                html!{
                                    <tr class={ if selected { "selected" } else { "" } }>
                                        <td>{ &e.at }</td>
                                        <td><a href={ tx_url } target="_blank">{ &e.tx[..7] }</a></td>
                                        <td>{ "invoke" }</td>
                                        <td></td>
                                        <td>{ &c_id[..7] }</td>
                                        <td><button onclick={scope.callback(move |_| HistoryCompMsg::SelectEvent(e.clone()))}>{ "view" }</button></td>
                                    </tr>
                                }
                            },
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
