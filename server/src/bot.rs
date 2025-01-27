use _BotProps::bot_item;
use serde::{ Deserialize, Serialize };
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use server_vars::types::bot::BotItem;

#[derive(Properties, PartialEq)]
pub struct BotProps {
    pub bot_item: BotItem,
}
#[function_component(BotComponent)]
pub fn bot_component(bot_props: &BotProps) -> Html {
    let greet_input_ref = use_node_ref();

    let name = use_state(|| String::new());

    let greet_msg = use_state(|| String::new());
    {
        let greet_msg = greet_msg.clone();
        let name = name.clone();
        let name2 = name.clone();
        use_effect_with(name2, move |_| {
            spawn_local(async move {
                if name.is_empty() {
                    return;
                }

                // let args = serde_wasm_bindgen::to_value(&(GreetArgs { name: &*name })).unwrap();
                // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
                // let new_msg = invoke("greet", args).await.as_string().unwrap();
                // greet_msg.set(new_msg);
            });

            || {}
        });
    }

    let greet = {
        let name = name.clone();
        let greet_input_ref = greet_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            name.set(greet_input_ref.cast::<web_sys::HtmlInputElement>().unwrap().value());
        })
    };

    html! {
        <div class="bot">
            <div class="flag" style="display: none; width: 30px;">
                {&bot_props.bot_item.flag}
            </div>
            <div class="name" style="display: inline-block; margin-left: 5px; width: 120px;">
                {&bot_props.bot_item.name}
            </div>
            <div class="cpu-brand" style="display: inline-block; margin-left: 5px; width: 120px;">
                {&bot_props.bot_item.cpu_brand}
            </div>
            <div class="ram" style="display: inline-block; margin-left: 5px; width: 80px;">
                {&bot_props.bot_item.ram}
            </div>
            <div class="ping" style="display: inline-block; margin-left: 5px; width: 80px;">
                {&bot_props.bot_item.ping}
            </div>
            <div class="connected-uptime" style="display: inline-block; margin-left: 5px; width: 120px;">
                {&bot_props.bot_item.join_date}
            </div>
            <div class="system-uptime" style="display: inline-block; margin-left: 5px; width: 120px;">
                {&bot_props.bot_item.system_boot_time}
            </div>
            <div class="region" style="display: inline-block; margin-left: 5px; width: 80px;">
                {&bot_props.bot_item.region}
            </div>
            <div class="os-info" style="display: inline-block; margin-left: 5px; width: 120px;">
                {&bot_props.bot_item.os_info}
            </div>
            <div class="active-window" style="display: inline-block; margin-left: 5px; width: 120px;">
                {&bot_props.bot_item.active_window}
            </div>
        </div>
    }
}
