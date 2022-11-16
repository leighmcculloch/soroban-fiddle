mod horizonapi;
mod invoke;
mod stream;

use invoke::invoke;
use stream::{collect, Event, EventBody, Order};

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
    events: Vec<stream::Event>,
    display: Option<String>,
    invoke_result: Option<String>,
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
            collect(HORIZON_BASE_URL, Order::Desc, move |event| {
                link.send_message(Msg::Event(event));
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
