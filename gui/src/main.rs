use std::{env, path::PathBuf};

use druid::{
    AppLauncher, Color, Data, Env, EventCtx, Lens, LocalizedString, PlatformError, Widget,
    WidgetExt, WindowDesc,
    platform_menus::win::file::exit,
    widget::{Button, Container, Either, Flex, Label, List, Padding, Scroll},
};
use rfd::FileDialog;
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
struct AppState {
    selected_folders: Arc<Vec<String>>,
    page: Page,
}

#[derive(Clone, Data, PartialEq)]
enum Page {
    PreOp,
    PostOp,
}

// const WINDOW_WIDTH: f64 = 500.0;
// const WINDOW_HEIGHT: f64 = 600.0;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        /* .with_min_size((WINDOW_WIDTH, WINDOW_HEIGHT)) */;
    let initial_state = AppState {
        selected_folders: Arc::new(Vec::new()),
        page: Page::PreOp,
    };
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
}

fn ui_builder() -> impl Widget<AppState> {
    Either::new(
        |data: &AppState, _env| data.page == Page::PreOp,
        pre_op_page(),
        post_op_page(),
    )
}

fn pre_op_page() -> impl Widget<AppState> {
    let button = Button::new("Select Folders").on_click(|_ctx, data: &mut AppState, _env| {
        if let Some(folders) = FileDialog::new()
            .set_directory(env::current_dir().unwrap())
            .pick_folders()
        {
            data.selected_folders = Arc::new(
                folders
                    .into_iter()
                    .map(|f| f.display().to_string())
                    .collect(),
            );
        }
    });

    /* let folder_list = Scroll::new(
        Flex::column()
            .with_child(Label::new(|data: &AppState, _env: &Env| {
                if data.selected_folders.is_empty() {
                    "".to_string()
                } else {
                    data.selected_folders.join("\n")
                }
            }))
            .padding(5.0),
    ); */
    let folder_list = Scroll::new(
        List::new(|| {
            Button::new(|item: &String, _env: &Env| item.clone())
                .on_click(|_ctx, item: &mut String, _env| println!("{}", item))
        })
        .lens(AppState::selected_folders),
    );

    let folder_container = Container::new(folder_list)
        .border(Color::WHITE, 2.0)
        .rounded(10.0)
        .expand_width()
        .padding(5.0);

    let run_button =
        Button::new("Run scan").on_click(|ctx: &mut EventCtx, data: &mut AppState, _env| {
            for folder in data.selected_folders.iter() {
                println!("{:?}", folder);
            }
            data.page = Page::PostOp;
            ctx.request_update();
        });

    Flex::column()
        .with_child(button)
        .with_child(Label::new("Selected Folders:"))
        .with_child(folder_container)
        .with_child(run_button)
}

fn post_op_page() -> impl Widget<AppState> {
    Flex::column()
        .with_child(Label::new("You did it"))
        .with_child(Button::new("Exit").on_click(|ctx, _, _| ctx.window().close()))
}
