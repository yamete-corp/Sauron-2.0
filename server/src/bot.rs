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
        <div class="flag">
          {bot_props.bot_item.flag.clone()}
        </div>
        <div class="name">
            {bot_props.bot_item.name.clone()}
        </div>
        <div class="cpu-brand">
            {bot_props.bot_item.cpu_brand.clone()}
        </div>
        <div class="ram">
            {bot_props.bot_item.ram.clone()}
        </div>
        <div class="ping">
            {bot_props.bot_item.ping.clone()}
        </div>
        <div class="connected-uptime">
            {bot_props.bot_item.join_date.clone()}
        </div>
        <div class="system-uptime">
            {bot_props.bot_item.system_boot_time.clone()}
        </div>
        <div class="region">
            {bot_props.bot_item.region.clone()}
        </div>
        <div class="os-info">
            {bot_props.bot_item.os_info.clone()}
        </div>
        <div class="active-window">
            {bot_props.bot_item.active_window.clone()}
        </div>
    </div>
    }
}
