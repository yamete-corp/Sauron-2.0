use std::{ collections::HashMap, vec };

use serde::{ Deserialize, Serialize };
use server_vars::types::bot::BotItem;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::*;
use crate::bot::BotComponent;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}
#[derive(Serialize, Deserialize)]
struct FetchBotsArgs {
    filter_ids: Vec<String>,
}

#[function_component(App)]
pub fn app() -> Html {
    // let loaded_bots: UseStateHandle<HashMap<String, BotItem>> = use_state(|| HashMap::new());
    // let loaded_bots_clone = loaded_bots.clone();
    // {
    //     use_interval(move || {
    //         let value = loaded_bots_clone.clone();
    //         spawn_local(async move {
    //             let args = serde_wasm_bindgen
    //                 ::to_value(
    //                     &(FetchBotsArgs {
    //                         filter_ids: value.keys().cloned().collect(),
    //                     })
    //                 )
    //                 .unwrap();

    //             let new_bots_string = invoke("get_bot_items", args).await.as_string().unwrap();
    //             let new_bots: HashMap<String, BotItem> = serde_json
    //                 ::from_str(&new_bots_string)
    //                 .unwrap();

    //             let updated_bots: HashMap<String, BotItem> = (*value)
    //                 .clone()
    //                 .into_iter()
    //                 .chain(new_bots.into_iter())
    //                 .collect();

    //             value.set(updated_bots);
    //         });
    //     }, 1000);
    // }
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

                let args = serde_wasm_bindgen::to_value(&(GreetArgs { name: &*name })).unwrap();
                // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
                let new_msg = invoke("greet", args).await.as_string().unwrap();
                greet_msg.set(new_msg);
            });

            || {}
        });
    }
    // let bot_item = BotItem {
    //     id: "12345".to_string(),
    //     flag: "<script>alert('XSS')</script>".to_string(), // XSS test
    //     name: "John Doe <script>alert('XSS')</script>".to_string(), // XSS test
    //     cpu_brand: "Intel Core i7 <img src='non-existent-image.jpg' onerror='alert(\"XSS\")'>".to_string(), // XSS test
    //     ram: "16 GB <iframe src='https://example.com'></iframe>".to_string(), // XSS test
    //     ping: "<a href='https://example.com'>example.com</a>".to_string(), // link injection test
    //     join_date: "10 days <script>location.href='https://example.com';</script>".to_string(), // XSS test
    //     system_boot_time: 30, // XSS test
    //     region: "US <img src='https://example.com/image.jpg' onerror='alert(\"XSS\")'>".to_string(), // XSS test
    //     os_info: "Windows 10 <script>document.write('Hello World!');</script>".to_string(), // XSS test
    //     active_window: "Google Chrome <iframe src='https://example.com'></iframe>".to_string(), // XSS test
    //     mib_config: None,
    //     client_instances: vec![],
    //     loader_instances: vec![],
    // };

    let greet = {
        let name = name.clone();
        let greet_input_ref = greet_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            name.set(greet_input_ref.cast::<web_sys::HtmlInputElement>().unwrap().value());
        })
    };

    html! {
        <main class="container">
            <h1>{"MAIN"}</h1>

            <div class="row">
                <a href="https://tauri.app" target="_blank">
                    <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://yew.rs" target="_blank">
                    <img src="public/yew.png" class="logo yew" alt="Yew logo"/>
                </a>
            </div>
            <p>{"Click on the Tauri and Yew logos to learn more."}</p>

            <form class="row" onsubmit={greet}>
                <input id="greet-input" ref={greet_input_ref} placeholder="Enter a name..." />
                <button type="submit">{"Greet"}</button>
            </form>
            <p>{ &*greet_msg }</p> 
            // <div>
            //     <BotComponent bot_item={{bot_item}} />
            // </div>
            <div>
            // {loaded_bots.values().cloned().map(|bot_item| {
            //     html! {
            //         <BotComponent bot_item={bot_item.clone()} />
            //     }
            // }).collect::<Html>()}
        </div>
        </main>
    }
}
