use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State, Url};

pub const GROUP_KNOWLEDGEBASE_LAUNCH_EVENT: &str = "sdkwork://knowledgebase/group-launch";

const LAUNCH_TICKET_PREFIX: &str = "gklt_";
const LAUNCH_TICKET_PAYLOAD_LENGTH: usize = 43;

#[derive(Clone, Debug, Serialize)]
pub struct GroupKnowledgebaseLaunchEvent {
    pub route: String,
}

#[derive(Default)]
pub struct GroupKnowledgebaseLaunchState(Mutex<Option<String>>);

impl GroupKnowledgebaseLaunchState {
    pub fn new() -> Self {
        Self::default()
    }

    fn remember(&self, route: String) {
        if let Ok(mut pending) = self.0.lock() {
            *pending = Some(route);
        }
    }

    fn take(&self) -> Result<Option<String>, String> {
        self.0
            .lock()
            .map_err(|_| "group knowledge base launch state is unavailable".to_string())
            .map(|mut pending| pending.take())
    }
}

fn is_valid_launch_ticket(ticket: &str) -> bool {
    ticket.len() == LAUNCH_TICKET_PREFIX.len() + LAUNCH_TICKET_PAYLOAD_LENGTH
        && ticket.starts_with(LAUNCH_TICKET_PREFIX)
        && ticket[LAUNCH_TICKET_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

pub fn parse_group_knowledgebase_launch_url(raw_url: &str) -> Option<String> {
    let url = Url::parse(raw_url).ok()?;
    if url.scheme() != "sdkwork-knowledgebase"
        || url.host_str() != Some("group-launch")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }

    let ticket = url.path().strip_prefix('/')?;
    if ticket.contains('/') || !is_valid_launch_ticket(ticket) {
        return None;
    }

    Some(ticket.to_string())
}

fn build_group_knowledgebase_route(ticket: &str) -> Option<String> {
    is_valid_launch_ticket(ticket).then(|| format!("/group-launch#ticket={ticket}"))
}

fn resolve_group_knowledgebase_launch_route(raw_url: &str) -> Option<String> {
    parse_group_knowledgebase_launch_url(raw_url)
        .and_then(|ticket| build_group_knowledgebase_route(ticket.as_str()))
}

fn resolve_group_knowledgebase_launch_route_urls<'a, I>(urls: I) -> Option<String>
where
    I: IntoIterator<Item = &'a Url>,
{
    urls.into_iter()
        .find_map(|url| resolve_group_knowledgebase_launch_route(url.as_str()))
}

pub fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn route_group_knowledgebase_launch_route(app: &AppHandle, route: String) -> bool {
    if let Some(state) = app.try_state::<GroupKnowledgebaseLaunchState>() {
        state.remember(route.clone());
    }
    // The reusable main window is the full standalone Knowledgebase window.
    // It never derives a label or title from ticket/group data.
    focus_main_window(app);
    app.emit_to(
        "main",
        GROUP_KNOWLEDGEBASE_LAUNCH_EVENT,
        GroupKnowledgebaseLaunchEvent { route },
    )
    .is_ok()
}

pub fn route_group_knowledgebase_launch(app: &AppHandle, raw_url: &str) -> bool {
    let Some(route) = resolve_group_knowledgebase_launch_route(raw_url) else {
        return false;
    };

    route_group_knowledgebase_launch_route(app, route)
}

pub fn route_group_knowledgebase_launch_urls<'a, I>(app: &AppHandle, urls: I) -> bool
where
    I: IntoIterator<Item = &'a Url>,
{
    let Some(route) = resolve_group_knowledgebase_launch_route_urls(urls) else {
        return false;
    };

    route_group_knowledgebase_launch_route(app, route)
}

#[tauri::command]
pub fn take_pending_group_knowledgebase_launch(
    state: State<'_, GroupKnowledgebaseLaunchState>,
) -> Result<Option<GroupKnowledgebaseLaunchEvent>, String> {
    let route = state.take()?;
    Ok(route.map(|route| GroupKnowledgebaseLaunchEvent { route }))
}

pub fn route_group_knowledgebase_launch_args(app: &AppHandle, args: &[String]) -> bool {
    args.iter()
        .any(|arg| route_group_knowledgebase_launch(app, arg.as_str()))
}

#[cfg(test)]
mod tests {
    use super::{
        build_group_knowledgebase_route, parse_group_knowledgebase_launch_url,
        resolve_group_knowledgebase_launch_route, resolve_group_knowledgebase_launch_route_urls,
        GroupKnowledgebaseLaunchState,
    };
    use tauri::Url;

    fn valid_ticket() -> String {
        format!("gklt_{}", "a".repeat(43))
    }

    #[test]
    fn accepts_exact_group_launch_url() {
        let ticket = valid_ticket();
        assert_eq!(
            parse_group_knowledgebase_launch_url(
                format!("sdkwork-knowledgebase://group-launch/{ticket}").as_str(),
            ),
            Some(ticket)
        );
    }

    #[test]
    fn rejects_context_and_uri_extensions() {
        for url in [
            "sdkwork-knowledgebase://group-launch/opaque?groupId=123",
            "sdkwork-knowledgebase://group-launch/opaque#ticket=other",
            "sdkwork-knowledgebase://group-launch/opaque/extra",
            "sdkwork-knowledgebase://other/opaque",
            "sdkwork-knowledgebase://group-launch/opaque%2Fextra",
            "sdkwork-knowledgebase://group-launch/opaque ticket",
        ] {
            assert_eq!(parse_group_knowledgebase_launch_url(url), None, "{url}");
        }
    }

    #[test]
    fn route_keeps_ticket_in_fragment_only() {
        let ticket = valid_ticket();
        assert_eq!(
            build_group_knowledgebase_route(ticket.as_str()),
            Some(format!("/group-launch#ticket={ticket}"))
        );
    }

    #[test]
    fn retains_a_valid_cold_start_route_until_the_renderer_consumes_it() {
        let ticket = valid_ticket();
        let state = GroupKnowledgebaseLaunchState::new();
        let invalid_url = Url::parse("sdkwork-knowledgebase://group-launch/gklt_short?tenant=1")
            .expect("invalid launch candidate must still be a URL");
        let valid_url =
            Url::parse(format!("sdkwork-knowledgebase://group-launch/{ticket}").as_str())
                .expect("valid launch URL must parse");
        let route = resolve_group_knowledgebase_launch_route_urls([&invalid_url, &valid_url]);

        state.remember(route.expect("valid group launch URL must create a route"));

        assert_eq!(
            state.take().expect("launch state lock must be available"),
            Some(format!("/group-launch#ticket={ticket}"))
        );
        assert_eq!(
            state.take().expect("launch state lock must be available"),
            None
        );
    }

    #[test]
    fn refuses_to_retain_an_invalid_cold_start_url() {
        let state = GroupKnowledgebaseLaunchState::new();
        assert_eq!(
            resolve_group_knowledgebase_launch_route(
                "sdkwork-knowledgebase://group-launch/gklt_short?tenant=1",
            ),
            None
        );
        let invalid_url = Url::parse("sdkwork-knowledgebase://group-launch/gklt_short?tenant=1")
            .expect("invalid launch candidate must still be a URL");
        assert_eq!(
            resolve_group_knowledgebase_launch_route_urls([&invalid_url]),
            None
        );
        assert_eq!(
            state.take().expect("launch state lock must be available"),
            None
        );
    }
}
