#![allow(unused_braces)]

use log::{trace, Level};
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::HashChangeEvent;

pub(crate) mod global;
pub(crate) mod pages;
pub(crate) mod ui_resources;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn main(parent_id: Option<String>) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    match console_log::init_with_level(Level::Trace) {
        Ok(_) => (),
        Err(e) => trace!("{:?}", e),
    }

    let gizmo = Gizmo::from(App { route: Route::Home });
    let view = View::from(gizmo.view_builder());
    if let Some(id) = parent_id {
        let parent = utils::document().get_element_by_id(&id).unwrap();
        view.run_in_container(&parent)
    } else {
        view.run()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Route {
    Home,
    Settings,
    Profile {
        username: String,
        is_favourites: bool,
    },
}

impl TryFrom<&str> for Route {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        trace!("route try_from: {}", s);

        let hash_split = s.split("#").collect::<Vec<_>>();
        let after_hash = match hash_split.as_slice() {
            [_, after] => Ok(after),
            _ => Err(format!("route must have a hash: {}", s)),
        }?;

        let paths: Vec<&str> = after_hash.split("/").collect::<Vec<_>>();
        trace!("route paths: {:?}", paths);

        match paths.as_slice() {
            [""] => Ok(Route::Home),
            ["", ""] => Ok(Route::Home),
            ["", "settings"] => Ok(Route::Settings),
            ["", "profile", username] => Ok(Route::Profile {
                username: username.to_string(),
                is_favourites: false,
            }),
            ["", "profile", username, "favorites"] => Ok(Route::Profile {
                username: username.to_string(),
                is_favourites: true,
            }),
            r => Err(format!("unsupported route: {:?}", r)),
        }
    }
}

impl From<Route> for String {
    fn from(route: Route) -> Self {
        match route {
            Route::Home => "#/".into(),
            Route::Settings => "#/settings".into(),
            Route::Profile {
                username,
                is_favourites,
            } => {
                if is_favourites {
                    format!("#profile/{}/favourites", username)
                } else {
                    format!("#profile/{}", username)
                }
            }
        }
    }
}

impl From<&Route> for ViewBuilder<HtmlElement> {
    fn from(route: &Route) -> Self {
        match route {
            Route::Home => pages::root(),
            Route::Settings => builder! {
                <main>
                    <h1>"Update your settings"</h1>
                </main>
            },
            Route::Profile {
                username,
                is_favourites,
            } => builder! {
                <main>
                    <h1>{username}"'s Profile"</h1>
                    {
                        if *is_favourites {
                            Some(builder! {
                                <h2>"Favourites"</h2>
                            })
                        }else {
                            None
                        }
                    }
                </main>
            },
        }
    }
}

impl From<&Route> for View<HtmlElement> {
    fn from(route: &Route) -> Self {
        ViewBuilder::from(route).into()
    }
}

impl Route {
    pub fn nav_home_class(&self) -> String {
        match self {
            Route::Home => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_settings_class(&self) -> String {
        match self {
            Route::Settings { .. } => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_profile_class(&self) -> String {
        match self {
            Route::Profile { .. } => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }
}

struct App {
    route: Route,
}

#[derive(Clone)]
enum AppModel {
    HashChange(String),
}

#[derive(Clone)]
enum AppView {
    PatchPage(Patch<View<HtmlElement>>),
    Error(String),
}

impl AppView {
    fn error(&self) -> Option<String> {
        match self {
            AppView::Error(msg) => Some(msg.clone()),
            _ => None,
        }
    }

    fn patch_page(&self) -> Option<Patch<View<HtmlElement>>> {
        match self {
            AppView::PatchPage(patch) => Some(patch.clone()),
            _ => None,
        }
    }
}

impl Component for App {
    type DomNode = HtmlElement;
    type ModelMsg = AppModel;
    type ViewMsg = AppView;

    fn update(
        &mut self,
        msg: &Self::ModelMsg,
        tx_view: &Transmitter<Self::ViewMsg>,
        sub: &Subscriber<Self::ModelMsg>,
    ) {
        match msg {
            AppModel::HashChange(hash) => match Route::try_from(hash.as_str()) {
                Err(msg) => tx_view.send(&AppView::Error(msg)),
                Ok(route) => {
                    if route != self.route {
                        let view = View::from(ViewBuilder::from(&route));
                        self.route = route;
                        tx_view.send(&AppView::PatchPage(Patch::Replace {
                            index: 0,
                            value: view,
                        }));
                    }
                }
            },
        }
    }

    fn view(
        &self,
        tx: &Transmitter<Self::ModelMsg>,
        rx: &Receiver<Self::ViewMsg>,
    ) -> ViewBuilder<Self::DomNode> {
        let username: String = "Reasonable-Human".into();

        builder! {
            <slot
                window:hashchange=tx.contra_filter_map(|ev: &Event| {
                    let hev = ev.dyn_ref::<HashChangeEvent>().unwrap().clone();
                    let hash = hev.new_url();
                    Some(AppModel::HashChange(hash))
                })
                patch:children=rx.branch_filter_map(AppView::patch_page)>

                /*<nav>
                    <ul>
                    <li class=self.route.nav_home_class()>
                        <a href=String::from(Route::Home)>"Home"</a>
                    </li>
                    <li class=self.route.nav_settings_class()>
                        <a href=String::from(Route::Settings)>"Settings"</a>
                    </li>


                    <li class=self.route.nav_settings_class()>
                        <a href=String::from(Route::Profile {
                            username: username.clone(),
                            is_favourites: true,
                        })>{format!("{} Profile", username)}</a>
                    </li>
                    </ul>
                </nav>
                <pre>{rx.branch_filter_map(AppView::error)}</pre>*/
                {ViewBuilder::from(&self.route)}

            </slot>
        }
    }
}

#[derive(Debug)]
pub struct PageMetadata {
    title: String,
    description: Option<String>,
    footer: Copyright,
}
#[derive(Debug)]
pub struct Copyright {
    symbol: &'static str,
    date: String,
    institution: &'static str,
}
