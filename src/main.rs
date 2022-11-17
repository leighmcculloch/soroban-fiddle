use ui::{history::SelectedEvent, invoke_info::InvokeInfoComp};
use yew::{
    prelude::{html, Component, Context, Html},
    start_app,
};

mod horizonapi;
mod stream;
mod ui;
mod vm;

use stream::EventBody;

use crate::ui::contract_info::ContractInfoComp;
use crate::ui::event_info::EventInfoComp;
use crate::ui::history::HistoryComp;
use crate::ui::invoke::InvokeComp;

const HORIZON_BASE_URL: &str = "https://horizon-futurenet.stellar.org";

fn main() {
    start_app::<App>();
}

#[derive(Default)]
struct App {
    selected_event: Option<SelectedEvent>,
}

enum AppMsg {
    SelectEvent(SelectedEvent),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::SelectEvent(e) => {
                self.selected_event = Some(e);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = ctx.link();
        let onevent = { scope.callback(AppMsg::SelectEvent) };
        html! {
            <>
            <div class="header">{ "Soroban Fiddle – data from FutureNet (" }<a href="https://soroban.stellar.org">{ "soroban.stellar.org" }</a>{ ")" }</div>
            <div class="columns">
                <div class="left">
                    <HistoryComp {onevent} />
                </div>
                <div class="right">
                {
                    if let Some(e) = &self.selected_event {
                        html!{
                            <>
                                <EventInfoComp event={e.event.clone()} />
                                {
                                    match &e.event.body {
                                        EventBody::Invocation(i) => html! {
                                            <>
                                                <InvokeInfoComp invocation={i.clone()} />
                                            </>
                                        },
                                        EventBody::Deployment(c) => html! {
                                            <>
                                                <ContractInfoComp contract={c.clone()} />
                                                <InvokeComp contract={c.clone()} event={e.event.clone()} related_events={e.related.clone()} />
                                            </>
                                        },
                                    }
                                }
                            </>
                        }
                    } else {
                        html!()
                    }
                }
                </div>
            </div>
            <div class="footer"><a href="https://github.com/leighmcculloch/soroban-fiddle">{ "open source" }</a></div>
            </>
        }
    }
}
